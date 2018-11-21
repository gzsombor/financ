use calamine::{DataType, Range};
use external_models::{ExternalTransaction, SheetFormat};
use sheets::{cell_to_date, cell_to_float, cell_to_iso_date, cell_to_string};
use utils::extract_date;

struct OtpFormat;
struct GranitFormat;

pub fn create_format(name: Option<String>) -> Option<Box<SheetFormat>> {
    if let Some(format_name) = name {
        match format_name.to_lowercase().as_ref() {
            "otp" => Some(Box::new(OtpFormat {})),
            "granit" => Some(Box::new(GranitFormat {})),
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
                let parsed_date = extract_date(descrip.clone());
                ExternalTransaction {
                    date: cell_to_date(&row[2]),
                    booking_date: cell_to_date(&row[3]),
                    amount: cell_to_float(&row[4]),
                    category: cell_to_string(&row[1]),
                    description: descrip,
                    other_account: cell_to_string(&row[6]),
                    other_account_name: cell_to_string(&row[7]),
                    textual_date: parsed_date,
                }
            })
            .collect()
    }
}

fn is_float(dt: &DataType) -> bool {
    match dt {
        DataType::Float(_) => true,
        _ => false,
    }
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

fn cleanup_string(input:&str) -> String {
    let casefix = if input.to_uppercase() == input {
        input.to_lowercase()
    } else {
        input.to_owned()
    };
    casefix.replace("A'","Á").replace("I'", "Í").replace("E'","É").replace("O'","Ó").replace("U'", "Ú").replace("U:", "Ü").replace("O:", "Ö")
        .replace("a'","á").replace("i'", "í").replace("e'","é").replace("o'","ó").replace("u'", "ú").replace("u:", "ü").replace("o:", "ö")
}

impl SheetFormat for GranitFormat {
    fn parse_sheet(&self, range: &Range<DataType>) -> Vec<ExternalTransaction> {
        range
            .rows()
            .filter(|row| is_float(&row[1]))
            .map(|row| {
                let date = cell_to_iso_date(&row[4]);
                let other_account_name =
                    cell_to_string(&row[7]).or_else(|| cell_to_string(&row[9])).map(|name| cleanup_string(&name));
                let comment = cell_to_string(&row[11]);
                ExternalTransaction {
                    date,
                    booking_date: None,
                    amount: cell_to_float(&row[1]),
                    category: cell_to_string(&row[6]),
                    description: concat(&other_account_name, &comment),
                    other_account: cell_to_string(&row[8]),//.or_else(|| cell_to_string(&row[10])),
                    other_account_name,
                    textual_date: None,
                }
            })
            .collect()
    }
}
