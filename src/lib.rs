use serde_json;
use serde_json::Value;

pub struct PriceData {
    pub price: f64,
    pub timestamp: u64,
}

pub struct RegressionDataSetTempEntry {
    price: f64,
    price_small: f64,
    begin_timestamp: f64,
    timestamp: f64,
    timestamp_small: f64,
    time_value: f64,
    time_square: f64,
}

pub struct RegressionDatasetFullEntry {
    price: f64,
    price_small: f64,
    begin_timestamp: f64,
    timestamp: f64,
    timestamp_small: f64,
    time_value: f64,
    time_square: f64,
    sum_price_small: f64,
    sum_time: f64,
    sum_time_value: f64,
    sum_time_square: f64,
    regression_a: f64,
    regression_b: f64,
    regression_value: f64,
    regression_a_abs: f64,
    regression_b_half: f64,
}

pub fn calculate_initial_regression(price_dataset: Vec<PriceData>) -> Vec<RegressionDatasetFullEntry> {
    let temp_dataset: Vec<RegressionDataSetTempEntry> = make_basic_regression_dataset(price_dataset, None);
    let mut dataset: Vec<RegressionDatasetFullEntry> = Vec::new();

    let mut sum_price_small: f64 = 0.0;
    let mut sum_time: f64 = 0.0;
    let mut sum_time_value: f64 = 0.0;
    let mut sum_time_square: f64 = 0.0;

    for entry in temp_dataset.iter() {
        sum_price_small += entry.price_small;
        sum_time += entry.timestamp_small;
        sum_time_value += entry.time_value;
        sum_time_square += entry.time_square;
    }

    let regression_a: f64 = calculate_regression_a(sum_price_small, sum_time_square, sum_time, sum_time_value, temp_dataset.len() as f64);
    let regression_b: f64 = calculate_regression_b(sum_price_small, sum_time_square, sum_time, sum_time_value, temp_dataset.len() as f64);
    let regression_value: f64 = calculate_regression_value(regression_a, regression_b, temp_dataset[temp_dataset.len() - 1].timestamp_small);
    let regression_a_abs: f64 = calculate_regression_a_abs(regression_a, regression_b, temp_dataset[0].begin_timestamp);
    let regression_b_half: f64 = calculate_initial_regression_b_half(regression_b);

    for entry in temp_dataset.iter() {
        dataset.push(RegressionDatasetFullEntry {
            price: entry.price,
            price_small: entry.price_small,
            begin_timestamp: entry.begin_timestamp,
            timestamp: entry.timestamp,
            timestamp_small: entry.timestamp_small,
            time_value: entry.time_value,
            time_square: entry.time_square,
            sum_price_small,
            sum_time,
            sum_time_value,
            sum_time_square,
            regression_a,
            regression_b,
            regression_value,
            regression_a_abs,
            regression_b_half,
        })
    }

    dataset
}

pub fn update_regression_dataset(mut regression_dataset: Vec<RegressionDatasetFullEntry>, price_dataset: Vec<PriceData>, regression_length_in_seconds: f64) -> Vec<RegressionDatasetFullEntry>{
    let temp_dataset: Vec<RegressionDataSetTempEntry> = make_basic_regression_dataset(price_dataset, Some(regression_dataset[regression_dataset.len() - 1].begin_timestamp));

    for i in 0..temp_dataset.len() {
        let mut sum_time: f64 = regression_dataset[regression_dataset.len() - 1].sum_time + temp_dataset[i].timestamp_small;
        let mut sum_price_small: f64 = regression_dataset[regression_dataset.len() - 1].sum_price_small + temp_dataset[i].price_small;
        let mut sum_time_value: f64 = regression_dataset[regression_dataset.len() - 1].sum_time_value + temp_dataset[i].time_value;
        let mut sum_time_square: f64 = regression_dataset[regression_dataset.len() - 1].sum_time_square + temp_dataset[i].time_square;

        let mut current_begin_timestamp: f64 = temp_dataset[0].begin_timestamp;
        let mut entry_to_be_deleted: Vec<usize> = Vec::new();
        for (usize, entry) in regression_dataset.iter().enumerate() {
            if temp_dataset[i].timestamp - entry.timestamp > regression_length_in_seconds {
                sum_time -= entry.timestamp_small;
                sum_price_small -= entry.price_small;
                sum_time_value -= entry.time_value;
                sum_time_square -= entry.time_square;
                entry_to_be_deleted.push(usize);
            } else {
                break;
            }
        }

        // Sort in descending order to avoid issues with shifting elements and remove the elements from the Vec
        entry_to_be_deleted.sort_by(|a, b| b.cmp(a));
        for &index in &entry_to_be_deleted {
            if index < regression_dataset.len() {
                regression_dataset.remove(index);
            } else {
                println!("Index {} out of bounds", index);
            }
        }

        let regression_a: f64 = calculate_regression_a(sum_price_small, sum_time_square, sum_time, sum_time_value, regression_dataset.len() as f64 + 1.0);
        let regression_b: f64 = calculate_regression_b(sum_price_small, sum_time_square, sum_time, sum_time_value, regression_dataset.len() as f64 + 1.0);
        let regression_value: f64 = calculate_regression_value(regression_a, regression_b, temp_dataset[i].timestamp_small);
        let regression_a_abs: f64 = calculate_regression_a_abs(regression_a, regression_b, temp_dataset[i].begin_timestamp);


        let check_timestamp: f64 = temp_dataset[i].timestamp - (0.5 * regression_length_in_seconds);
        let mut regression_b_half: f64 = 123456789.12;
        for entry in regression_dataset.iter() {
            regression_b_half = entry.regression_b;
            if entry.timestamp > check_timestamp {
                break;
            }
        }

        if regression_b_half == 123456789.12 {
            panic!("ILLEGAL");
        }

        regression_dataset.push(
            RegressionDatasetFullEntry {
                price: temp_dataset[i].price,
                price_small: temp_dataset[i].price_small,
                begin_timestamp: current_begin_timestamp,
                timestamp: temp_dataset[i].timestamp,
                timestamp_small: temp_dataset[i].timestamp_small,
                time_value: temp_dataset[i].time_value,
                time_square: temp_dataset[i].time_square,
                sum_price_small,
                sum_time,
                sum_time_value,
                sum_time_square,
                regression_a,
                regression_b,
                regression_value,
                regression_a_abs,
                regression_b_half,
            }
        )
    }

    regression_dataset
}


fn make_basic_regression_dataset(price_dataset: Vec<PriceData>, begin_timestamp: Option<f64>) -> Vec<RegressionDataSetTempEntry> {
    let mut result: Vec<RegressionDataSetTempEntry> = Vec::new();
    let mut begin_timestamp: f64 = begin_timestamp.unwrap_or(-0.1);

    for price_data in price_dataset {
        let price: f64 = price_data.price;
        let timestamp: f64 = price_data.timestamp as f64;
        if begin_timestamp == -0.1 {
            begin_timestamp = timestamp;
        }
        let price_small: f64 = price / 1000.0;
        let timestamp_small: f64 = (timestamp - begin_timestamp) / 100000.0;
        let time_value: f64 = timestamp_small * price_small;
        let time_square: f64 = f64::powf(timestamp_small, 2.0);
        result.push(RegressionDataSetTempEntry {
            price,
            price_small,
            begin_timestamp,
            timestamp,
            timestamp_small,
            time_value,
            time_square,
        });
    }

    result
}

fn calculate_regression_a(
    sum_price_small: f64, sum_time_square: f64, sum_time: f64, sum_time_value: f64, regression_len: f64,
) -> f64 {
    (((sum_price_small * sum_time_square) - (sum_time * sum_time_value)) / (regression_len * sum_time_square - f64::powf(sum_time, 2.0))) * 1000.0
}

fn calculate_regression_b(
    sum_price_small: f64, sum_time_square: f64, sum_time: f64, sum_time_value: f64, regression_len: f64,
) -> f64 {
    (((regression_len * sum_time_value) - (sum_time * sum_price_small)) / (regression_len * sum_time_square - f64::powf(sum_time, 2.0)) ) / 100.0
}

fn calculate_regression_value(regression_a: f64, regression_b: f64, time_stamp: f64) -> f64 {
    regression_a + ((regression_b * time_stamp) * 100000.0)
}

fn calculate_regression_a_abs(regression_a: f64, regression_b: f64, begin_timestamp: f64) -> f64 {
    regression_a - ((begin_timestamp / 100000.0) * regression_b)
}

fn calculate_initial_regression_b_half(regression_b: f64) -> f64 {
    regression_b
}