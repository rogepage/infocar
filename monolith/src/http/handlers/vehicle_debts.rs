use crate::application::plate_masker::PlateMasker;
use crate::domain::debt::errors::DomainError;
use crate::http::middleware::request_id::RequestId;
use crate::http::state::AppState;
use axum::Json;
use axum::body::Bytes;
use axum::extract::{Extension, State};
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use regex::Regex;
use serde_json::{Map, Value, json};
use std::sync::OnceLock;
use std::time::Instant;

fn plate_regex() -> &'static Regex {
    static RE: OnceLock<Regex> = OnceLock::new();
    RE.get_or_init(|| Regex::new(r"^[A-Z]{3}[0-9][A-Z0-9][0-9]{2}$|^[A-Z]{3}[0-9]{4}$").unwrap())
}

const ALLOWED_FIELDS: [&str; 2] = ["placa", "provider"];
const ALLOWED_PROVIDERS: [&str; 4] = ["rest", "soap", "ai", "csv"];

fn log_failed(
    state: &AppState,
    request_id: &str,
    plate: Option<&str>,
    error_type: &str,
    status_code: u16,
    reason: &str,
) {
    state.metrics.increment("vehicle_debt_error_total");
    let mut fields = Map::new();
    if let Some(p) = plate {
        fields.insert("plate".to_string(), Value::String(PlateMasker::mask(p)));
    }
    fields.insert("error_type".to_string(), Value::String(error_type.to_string()));
    fields.insert("status_code".to_string(), Value::from(status_code));
    fields.insert("reason".to_string(), Value::String(reason.to_string()));
    state.logger.warning(
        "vehicle_debt.failed",
        "vehicle_debt.failed",
        Some(request_id),
        fields,
    );
}

pub async fn post_vehicle_debts(
    State(state): State<AppState>,
    Extension(RequestId(request_id)): Extension<RequestId>,
    body: Bytes,
) -> Response {
    let payload: Value = match serde_json::from_slice(&body) {
        Ok(v) => v,
        Err(_) => {
            log_failed(&state, &request_id, None, "invalid_json", 400, "invalid_json");
            return (
                StatusCode::BAD_REQUEST,
                Json(json!({ "error": "A placa do veiculo e obrigatoria." })),
            )
                .into_response();
        }
    };

    let obj = match payload.as_object() {
        Some(o) => o,
        None => {
            log_failed(&state, &request_id, None, "invalid_json", 400, "expected_object");
            return (
                StatusCode::BAD_REQUEST,
                Json(json!({ "error": "A placa do veiculo e obrigatoria." })),
            )
                .into_response();
        }
    };

    // Prioridade 1: Detecção de campos desconhecidos
    let unrecognized: Vec<String> = obj
        .keys()
        .filter(|k| !ALLOWED_FIELDS.contains(&k.as_str()))
        .cloned()
        .collect();
    if !unrecognized.is_empty() {
        log_failed(
            &state,
            &request_id,
            obj.get("placa").and_then(|v| v.as_str()),
            "unknown_field",
            400,
            "unknown_field",
        );
        return (
            StatusCode::BAD_REQUEST,
            Json(json!({
                "error": "unknown_field",
                "unrecognized_fields": unrecognized
            })),
        )
            .into_response();
    }

    // Validação de placa ausente ou vazia
    let raw_plate = match obj.get("placa").and_then(|v| v.as_str()) {
        Some(p) if !p.trim().is_empty() => p.trim().to_uppercase(),
        _ => {
            log_failed(&state, &request_id, None, "missing_plate", 400, "missing_plate");
            return (
                StatusCode::BAD_REQUEST,
                Json(json!({ "error": "A placa do veiculo e obrigatoria." })),
            )
                .into_response();
        }
    };

    // Prioridade 0: Formato da placa
    if !plate_regex().is_match(&raw_plate) {
        log_failed(
            &state,
            &request_id,
            Some(&raw_plate),
            "invalid_plate",
            400,
            "invalid_plate",
        );
        return (
            StatusCode::BAD_REQUEST,
            Json(json!({ "error": "invalid_plate" })),
        )
            .into_response();
    }

    // Validação de provider opcional
    let provider_opt: Option<String> = match obj.get("provider") {
        None | Some(Value::Null) => None,
        Some(Value::String(s)) => {
            let trimmed = s.trim().to_lowercase();
            if trimmed.is_empty() {
                None
            } else if !ALLOWED_PROVIDERS.contains(&trimmed.as_str()) {
                log_failed(
                    &state,
                    &request_id,
                    Some(&raw_plate),
                    "invalid_provider",
                    400,
                    "invalid_provider",
                );
                return (
                    StatusCode::BAD_REQUEST,
                    Json(json!({
                        "error": "O provedor informado e invalido. Provedores permitidos: rest, soap, ai, csv."
                    })),
                )
                    .into_response();
            } else {
                Some(trimmed)
            }
        }
        Some(_) => {
            log_failed(
                &state,
                &request_id,
                Some(&raw_plate),
                "invalid_provider",
                400,
                "invalid_provider",
            );
            return (
                StatusCode::BAD_REQUEST,
                Json(json!({
                    "error": "O provedor informado e invalido. Provedores permitidos: rest, soap, ai, csv."
                })),
            )
                .into_response();
        }
    };

    let start = Instant::now();
    match state
        .use_case
        .execute(&raw_plate, provider_opt.as_deref(), Some(&request_id))
        .await
    {
        Ok(res) => {
            let duration_ms = start.elapsed().as_millis() as u64;
            state.metrics.increment("vehicle_debt_success_total");

            let mut ok_fields = Map::new();
            ok_fields.insert("plate".to_string(), Value::String(PlateMasker::mask(&raw_plate)));
            ok_fields.insert("provider".to_string(), Value::String(res.provider_used.clone()));
            ok_fields.insert("debts_count".to_string(), Value::from(res.calculated.debts.len()));
            ok_fields.insert("duration_ms".to_string(), Value::from(duration_ms));
            state.logger.info(
                "vehicle_debt.completed",
                "vehicle_debt.completed",
                Some(&request_id),
                ok_fields,
            );

            let debitos_json: Vec<Value> = res
                .calculated
                .debts
                .iter()
                .map(|d| {
                    json!({
                        "tipo": d.debt_type.as_str(),
                        "valor_original": d.original_amount.to_decimal(),
                        "valor_atualizado": d.updated_amount.to_decimal(),
                        "vencimento": d.due_date.format("%Y-%m-%d").to_string(),
                        "dias_atraso": d.days_overdue,
                    })
                })
                .collect();

            let opcoes_json: Vec<Value> = res
                .payment_simulation
                .options
                .iter()
                .map(|opt| {
                    let parcelas_json: Vec<Value> = opt
                        .credit_card
                        .installments
                        .iter()
                        .map(|inst| {
                            json!({
                                "quantidade": inst.quantity,
                                "valor_parcela": inst.installment_amount.to_decimal(),
                            })
                        })
                        .collect();

                    json!({
                        "tipo": opt.option_type,
                        "valor_base": opt.base_amount.to_decimal(),
                        "pix": {
                            "total_com_desconto": opt.pix.total_with_discount.to_decimal(),
                        },
                        "cartao_credito": {
                            "parcelas": parcelas_json,
                        }
                    })
                })
                .collect();

            (
                StatusCode::OK,
                Json(json!({
                    "placa": res.calculated.plate,
                    "debitos": debitos_json,
                    "resumo": {
                        "total_original": res.calculated.total_original.to_decimal(),
                        "total_atualizado": res.calculated.total_updated.to_decimal(),
                    },
                    "pagamentos": {
                        "opcoes": opcoes_json,
                    }
                })),
            )
                .into_response()
        }
        Err(DomainError::UnknownDebtType(debt_type)) => {
            log_failed(
                &state,
                &request_id,
                Some(&raw_plate),
                "unknown_debt_type",
                422,
                &debt_type,
            );
            (
                StatusCode::UNPROCESSABLE_ENTITY,
                Json(json!({
                    "error": "unknown_debt_type",
                    "type": debt_type,
                })),
            )
                .into_response()
        }
        Err(DomainError::AllProvidersUnavailable(_)) | Err(_) => {
            log_failed(
                &state,
                &request_id,
                Some(&raw_plate),
                "all_providers_unavailable",
                503,
                "all_providers_unavailable",
            );
            (
                StatusCode::SERVICE_UNAVAILABLE,
                Json(json!({
                    "error": "all_providers_unavailable",
                })),
            )
                .into_response()
        }
    }
}
