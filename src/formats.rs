use calamine::{DataType, Range};
use external_models::{ExternalTransaction, SheetFormat};
use sheets::{cell_to_date, cell_to_float, cell_to_string};
use utils::extract_date;

struct OtpFormat;

pub fn create_format(name: Option<String>) -> impl SheetFormat {
    OtpFormat {}
}

impl SheetFormat for OtpFormat {
    fn parse_sheet(&self, range: &Range<DataType>) -> Vec<ExternalTransaction> {
        /*        println!(
                    "Range starts : {:?} ends at {:?}",
                    range.start(),
                    range.end()
                );
        */
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
