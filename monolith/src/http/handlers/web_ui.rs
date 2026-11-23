use crate::domain::debt::errors::DomainError;
use crate::domain::debt::money::Money;
use crate::http::middleware::request_id::RequestId;
use crate::http::state::AppState;
use askama::Template;
use axum::Form;
use axum::extract::{Extension, State};
use axum::http::StatusCode;
use axum::response::{Html, IntoResponse, Response};
use regex::Regex;
use serde::Deserialize;
use std::sync::OnceLock;

fn ui_plate_regex() -> &'static Regex {
    static RE: OnceLock<Regex> = OnceLock::new();
    RE.get_or_init(|| Regex::new(r"^[A-Z]{3}[0-9][A-Z0-9][0-9]{2}$|^[A-Z]{3}[0-9]{4}$").unwrap())
}

pub fn format_brl(money: Money) -> String {
    let cents = money.amount_in_cents();
    let sign = if cents < 0 { "-" } else { "" };
    let abs = cents.unsigned_abs();
    let units = abs / 100;
    let frac = abs % 100;

    let units_str = units.to_string();
    let mut formatted_units = String::new();
    for (idx, ch) in units_str.chars().rev().enumerate() {
        if idx > 0 && idx % 3 == 0 {
            formatted_units.push('.');
        }
        formatted_units.push(ch);
    }
    let units_pt: String = formatted_units.chars().rev().collect();
    format!("{}R$ {},{:02}", sign, units_pt, frac)
}

#[derive(Debug)]
pub struct DebtViewModel {
    pub debt_type: String,
    pub days_overdue: i64,
    pub due_date_br: String,
    pub original_brl: String,
    pub updated_brl: String,
}

#[derive(Debug)]
pub struct InstallmentViewModel {
    pub quantity: u8,
    pub amount_brl: String,
}

#[derive(Debug)]
pub struct PaymentOptionViewModel {
    pub title: String,
    pub base_brl: String,
    pub pix_brl: String,
    pub installments: Vec<InstallmentViewModel>,
}

#[derive(Debug)]
pub struct LookupResultViewModel {
    pub plate: String,
    pub provider_used: String,
    pub debts: Vec<DebtViewModel>,
    pub total_original_brl: String,
    pub total_updated_brl: String,
    pub payment_options: Vec<PaymentOptionViewModel>,
}

#[derive(Template)]
#[template(path = "lookup.html")]
pub struct LookupTemplate {
    pub plate_input: String,
    pub error_message: Option<String>,
    pub result: Option<LookupResultViewModel>,
}

#[derive(Deserialize)]
pub struct LookupForm {
    pub placa: Option<String>,
}

fn render_template(tpl: LookupTemplate, status: StatusCode) -> Response {
    match tpl.render() {
        Ok(html) => (status, Html(html)).into_response(),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            format!("Template render error: {}", e),
        )
            .into_response(),
    }
}

pub async fn get_web_ui() -> Response {
    render_template(
        LookupTemplate {
            plate_input: String::new(),
            error_message: None,
            result: None,
        },
        StatusCode::OK,
    )
}

pub async fn post_web_ui(
    State(state): State<AppState>,
    Extension(RequestId(request_id)): Extension<RequestId>,
    Form(form): Form<LookupForm>,
) -> Response {
    let normalized = form.placa.unwrap_or_default().trim().to_uppercase();

    if normalized.is_empty() {
        return render_template(
            LookupTemplate {
                plate_input: normalized,
                error_message: Some("A placa do veículo é obrigatória.".to_string()),
                result: None,
            },
            StatusCode::OK,
        );
    }

    if !ui_plate_regex().is_match(&normalized) {
        return render_template(
            LookupTemplate {
                plate_input: normalized,
                error_message: Some("Placa inválida. Verifique o formato informado.".to_string()),
                result: None,
            },
            StatusCode::OK,
        );
    }

    match state
        .use_case
        .execute(&normalized, None, Some(&request_id))
        .await
    {
        Ok(res) => {
            let debts_vm = res
                .calculated
                .debts
                .iter()
                .map(|d| DebtViewModel {
                    debt_type: d.debt_type.as_str().to_string(),
                    days_overdue: d.days_overdue,
                    due_date_br: d.due_date.format("%d/%m/%Y").to_string(),
                    original_brl: format_brl(d.original_amount),
                    updated_brl: format_brl(d.updated_amount),
                })
                .collect();

            let options_vm = res
                .payment_simulation
                .options
                .iter()
                .map(|opt| {
                    let title = if opt.option_type == "TOTAL" {
                        "Pagamento Total dos Débitos".to_string()
                    } else if let Some(cat) = opt.option_type.strip_prefix("SOMENTE_") {
                        format!("Pagamento Parcial — {}", cat)
                    } else {
                        opt.option_type.clone()
                    };

                    let installments = opt
                        .credit_card
                        .installments
                        .iter()
                        .map(|inst| InstallmentViewModel {
                            quantity: inst.quantity,
                            amount_brl: format_brl(inst.installment_amount),
                        })
                        .collect();

                    PaymentOptionViewModel {
                        title,
                        base_brl: format_brl(opt.base_amount),
                        pix_brl: format_brl(opt.pix.total_with_discount),
                        installments,
                    }
                })
                .collect();

            render_template(
                LookupTemplate {
                    plate_input: normalized,
                    error_message: None,
                    result: Some(LookupResultViewModel {
                        plate: res.calculated.plate,
                        provider_used: res.provider_used,
                        debts: debts_vm,
                        total_original_brl: format_brl(res.calculated.total_original),
                        total_updated_brl: format_brl(res.calculated.total_updated),
                        payment_options: options_vm,
                    }),
                },
                StatusCode::OK,
            )
        }
        Err(DomainError::UnknownDebtType(t)) => render_template(
            LookupTemplate {
                plate_input: normalized,
                error_message: Some(format!(
                    "O sistema encontrou um tipo de débito não reconhecido ({}). Entre em contato com o suporte.",
                    t
                )),
                result: None,
            },
            StatusCode::OK,
        ),
        Err(_) => render_template(
            LookupTemplate {
                plate_input: normalized,
                error_message: Some(
                    "Não foi possível consultar os débitos no momento. Tente novamente em alguns instantes.".to_string(),
                ),
                result: None,
            },
            StatusCode::OK,
        ),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_format_brl() {
        assert_eq!(
            format_brl(Money::from_decimal("1500.00").unwrap()),
            "R$ 1.500,00"
        );
        assert_eq!(
            format_brl(Money::from_decimal("300.50").unwrap()),
            "R$ 300,50"
        );
        assert_eq!(format_brl(Money::from_decimal("0.00").unwrap()), "R$ 0,00");
    }
}
