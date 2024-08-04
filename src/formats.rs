use crate::external_models::{ExternalTransaction, SheetFormat};
use crate::sheets::{
    cell_to_date, cell_to_datetime, cell_to_decimal, cell_to_english_date, cell_to_german_date,
    cell_to_iso_date, cell_to_string,
};
use crate::utils::extract_date;
use calamine::{DataType, Range};

struct OtpFormat;
struct OtpFormat2020;
struct GranitFormat;
struct BankAustriaFormat;
struct TransferwiseFormat;
struct MagnetFormat;

pub fn create_format(name: &Option<String>) -> Option<Box<dyn SheetFormat>> {
    if let Some(ref format_name) = name {
        match format_name.to_lowercase().as_ref() {
            "otp" => Some(Box::new(OtpFormat {})),
            "otp2020" => Some(Box::new(OtpFormat2020 {})),
            "granit" => Some(Box::new(GranitFormat {})),
            "bankaustria" => Some(Box::new(BankAustriaFormat {})),
            "transferwise" => Some(Box::new(TransferwiseFormat {})),
            "magnet" => Some(Box::new(MagnetFormat {})),
            _ => None,
        }
    } else {
        Some(Box::new(OtpFormat {}))
    }
}

impl SheetFormat for OtpFormat {
    fn parse_sheet(&self, range: &Range<DataType>) -> Vec<ExternalTransaction> {
        range
            .rows()
            .filter(|row| row[0] != DataType::Empty)
            .map(|row| {
                let descrip = cell_to_string(&row[8]);
                let parsed_date = extract_date(&descrip);
                ExternalTransaction {
                    date: cell_to_date(&row[2]),
                    booking_date: cell_to_date(&row[3]),
                    amount: cell_to_decimal(&row[4]),
                    category: cell_to_string(&row[1]),
                    description: descrip,
                    other_account: cell_to_string(&row[6]),
                    other_account_name: cell_to_string(&row[7]),
                    textual_date: parsed_date,
                    transaction_fee: None,
                }
            })
            .collect()
    }
}

impl SheetFormat for OtpFormat2020 {
    fn parse_sheet(&self, range: &Range<DataType>) -> Vec<ExternalTransaction> {
        range
            .rows()
            .filter(|row| row[0] != DataType::Empty)
            .map(|row| {
                let spend_date = cell_to_datetime(&row[2]);
                let description = cell_to_string(&row[7]);
                let parsed_date = extract_date(&description);
                ExternalTransaction {
                    date: spend_date.map(|datetime| datetime.date()),
                    booking_date: cell_to_date(&row[3]),
                    amount: cell_to_decimal(&row[4]),
                    category: cell_to_string(&row[1]),
                    description,
                    other_account: cell_to_string(&row[5]),
                    other_account_name: cell_to_string(&row[6]),
                    textual_date: parsed_date,
                    transaction_fee: None,
                }
            })
            .collect()
    }
}

fn is_float(dt: &DataType) -> bool {
    matches!(dt, DataType::Float(_))
}

fn concat(first: &Option<String>, second: &Option<String>) -> Option<String> {
    match (first, second) {
        (Some(f), Some(snd)) => {
            let mut x = f.clone();
            x.push(' ');
            x.push_str(snd);
            Some(x)
        }
        (Some(f), None) => Some(f.clone()),
        (None, Some(f)) => Some(f.clone()),
        (_, _) => None,
    }
}

fn cleanup_string(input: &str) -> String {
    let casefix = if input.to_uppercase() == input {
        input.to_lowercase()
    } else {
        input.to_owned()
    };
    casefix
        .replace("A'", "Á")
        .replace("I'", "Í")
        .replace("E'", "É")
        .replace("O'", "Ó")
        .replace("U'", "Ú")
        .replace("U:", "Ü")
        .replace("O:", "Ö")
        .replace("a'", "á")
        .replace("i'", "í")
        .replace("e'", "é")
        .replace("o'", "ó")
        .replace("u'", "ú")
        .replace("u:", "ü")
        .replace("o:", "ö")
}

impl SheetFormat for GranitFormat {
    fn parse_sheet(&self, range: &Range<DataType>) -> Vec<ExternalTransaction> {
        range
            .rows()
            .filter(|row| is_float(&row[1]))
            .map(|row| {
                let date = cell_to_iso_date(&row[4]);
                let other_account_name = cell_to_string(&row[7])
                    .or_else(|| cell_to_string(&row[9]))
                    .map(|name| cleanup_string(&name));
                let comment = cell_to_string(&row[11]);
                //.map(|x| x.replace("****1683",""));
                //println!("Row is {:?} -> date {:?} comment {:?} other_name: {:?}", row, date, comment, other_account_name);
                ExternalTransaction {
                    date,
                    booking_date: None,
                    amount: cell_to_decimal(&row[1]),
                    category: cell_to_string(&row[6]),
                    description: concat(&other_account_name, &comment),
                    other_account: cell_to_string(&row[8]), //.or_else(|| cell_to_string(&row[10])),
                    other_account_name,
                    textual_date: None,
                    transaction_fee: None,
                }
            })
            .collect()
    }
}

impl SheetFormat for BankAustriaFormat {
    fn parse_sheet(&self, range: &Range<DataType>) -> Vec<ExternalTransaction> {
        range
            .rows()
            .skip(1)
            .filter(|row| is_float(&row[6]))
            .map(|row| {
                let date = cell_to_german_date(&row[1]);
                let booking_date = cell_to_german_date(&row[1]);
                let amount = cell_to_decimal(&row[6]).unwrap();
                let other_account = if amount.is_sign_negative() {
                    cell_to_string(&row[12])
                } else {
                    cell_to_string(&row[9])
                };
                ExternalTransaction {
                    date,
                    booking_date,
                    amount: Some(amount),
                    category: None,
                    description: cell_to_string(&row[3]).map(|s| s.trim().to_owned()),
                    other_account,
                    other_account_name: None,
                    textual_date: None,
                    transaction_fee: None,
                }
            })
            .collect()
    }
}

impl SheetFormat for TransferwiseFormat {
    fn parse_sheet(&self, range: &Range<DataType>) -> Vec<ExternalTransaction> {
        range
            .rows()
            .skip(1)
            .filter(|row| is_float(&row[2]))
            .map(|row| {
                let date = cell_to_english_date(&row[1]);
                let amount = cell_to_decimal(&row[2]);
                let other_account_name =
                    cell_to_string(&row[13]).or_else(|| cell_to_string(&row[11]));
                let other_account = cell_to_string(&row[12]);

                ExternalTransaction {
                    date,
                    booking_date: None,
                    amount,
                    category: None,
                    description: cell_to_string(&row[4]).map(|s| s.trim().to_owned()),
                    other_account,
                    other_account_name,
                    textual_date: None,
                    transaction_fee: cell_to_decimal(&row[14])
                        .filter(|value| value.is_sign_positive()),
                }
            })
            .collect()
    }
}

impl SheetFormat for MagnetFormat {
    fn parse_sheet(&self, range: &Range<DataType>) -> Vec<ExternalTransaction> {
        range
            .rows()
            .skip(1)
            .filter(|row| is_float(&row[6]))
            .map(|row| {
                let date = cell_to_date(&row[1]);
                let booking_date = cell_to_date(&row[2]);
                let amount = cell_to_decimal(&row[6]);
                let other_account = cell_to_string(&row[4]);
                let other_account_name = cell_to_string(&row[3]);
                let description = cell_to_string(&row[5]);

                ExternalTransaction {
                    date,
                    booking_date,
                    amount,
                    category: None,
                    description: concat(&other_account_name, &description),
                    other_account,
                    other_account_name,
                    textual_date: None,
                    transaction_fee: None,
                }
            })
            .collect()
    }
}
