use serde_json;
use serde_json::Value;

pub struct PriceData {
    pub price: f64,
    pub timestamp: u64,
}

pub struct RegressionData {
    pub price: f64,
    pub price_small: f64,
    pub begin_timestamp: f64,
    pub timestamp: f64,
    pub timestamp_small: f64,
    pub time_value: f64,
    pub time_square: f64,
    pub sum_price_small: f64,
    pub sum_time: f64,
    pub sum_time_value: f64,
    pub sum_time_square: f64,
    pub regression_a: f64,
    pub regression_b: f64,
    pub regression_value: f64,
    pub regression_a_abs: f64,
    pub regression_b_half: f64,
}

struct RegressionDataTemporary {
    pub price: f64,
    pub price_small: f64,
    pub begin_timestamp: f64,
    pub timestamp: f64,
    pub timestamp_small: f64,
    pub time_value: f64,
    pub time_square: f64,
}

// Function to calculate the initial regression dataset with given price data
pub fn calculate_initial_regression(price_dataset: Vec<PriceData>) -> Vec<RegressionData> {
    let temp_dataset: Vec<RegressionDataTemporary> = make_basic_regression_dataset(price_dataset, None);
    let mut dataset: Vec<RegressionData> = Vec::new();

    // Sum placeholders for further operations
    let mut sum_price_small: f64 = 0.0;
    let mut sum_time: f64 = 0.0;
    let mut sum_time_value: f64 = 0.0;
    let mut sum_time_square: f64 = 0.0;

    // Loop over each data point in temporary dataset and calculate essential sums
    for entry in temp_dataset.iter() {
        sum_price_small += entry.price_small;
        sum_time += entry.timestamp_small;
        sum_time_value += entry.time_value;
        sum_time_square += entry.time_square;
    }

    // Regression calculations
    let regression_a: f64 = calculate_regression_a(sum_price_small, sum_time_square, sum_time, sum_time_value, temp_dataset.len() as f64);
    let regression_b: f64 = calculate_regression_b(sum_price_small, sum_time_square, sum_time, sum_time_value, temp_dataset.len() as f64);
    let regression_value: f64 = calculate_regression_value(regression_a, regression_b, temp_dataset[temp_dataset.len() - 1].timestamp_small);
    let regression_a_abs: f64 = calculate_regression_a_abs(regression_a, regression_b, temp_dataset[0].begin_timestamp);
    let regression_b_half: f64 = calculate_initial_regression_b_half(regression_b);

    // Populate final dataset
    for entry in temp_dataset.iter() {
        dataset.push(RegressionData {
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
// Function to update the previously calculated regression dataset with the newly arrived price data
pub fn update_regression_dataset(mut regression_dataset: Vec<RegressionData>, price_dataset: Vec<PriceData>, regression_length_in_seconds: f64) -> Vec<RegressionData>{
    let temp_dataset: Vec<RegressionDataTemporary> = make_basic_regression_dataset(price_dataset, Some(regression_dataset[regression_dataset.len() - 1].begin_timestamp));

    for i in 0..temp_dataset.len() {
        let mut sum_time: f64 = regression_dataset[regression_dataset.len() - 1].sum_time + temp_dataset[i].timestamp_small;
        let mut sum_price_small: f64 = regression_dataset[regression_dataset.len() - 1].sum_price_small + temp_dataset[i].price_small;
        let mut sum_time_value: f64 = regression_dataset[regression_dataset.len() - 1].sum_time_value + temp_dataset[i].time_value;
        let mut sum_time_square: f64 = regression_dataset[regression_dataset.len() - 1].sum_time_square + temp_dataset[i].time_square;

        // The timestamp when the current regression data set begins is retrieved
        let mut current_begin_timestamp: f64 = temp_dataset[0].begin_timestamp;
        // An empty vector to store the indices of entries to be deleted is created
        let mut entry_to_be_deleted: Vec<usize> = Vec::new();
        // Iterating over each entry in the regression data set along with its index
        for (usize, entry) in regression_dataset.iter().enumerate() {
            // If the timestamp difference is greater than the specified regression length in seconds...
            if temp_dataset[i].timestamp - entry.timestamp > regression_length_in_seconds {
                // Removed entry's time-related data from their respective sums
                sum_time -= entry.timestamp_small;
                sum_price_small -= entry.price_small;
                sum_time_value -= entry.time_value;
                sum_time_square -= entry.time_square;
                // The indices of the entries to be deleted in the future are appended to the vector
                entry_to_be_deleted.push(usize);
            } else {
                // If the timestamp difference is less or equal to the regression length, break the loop.
                // As the rest of data points will have even smaller timestamp differences
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

        // Check if the timestamp minus half the regression length is less than the current timestamp
        let check_timestamp: f64 = temp_dataset[i].timestamp - (0.5 * regression_length_in_seconds);
        // Set the initial value of regression_b_half to a defined fake number
        let mut regression_b_half: f64 = 123456789.12;
        // Loop through entries in the regression_dataset, each entry's regression_b is assigned to regression_b_half
        // The loop breaks when an entry's timestamp is greater than the check_timestamp
        for entry in regression_dataset.iter() {
            regression_b_half = entry.regression_b;
            if entry.timestamp > check_timestamp {
                //break the loop if the condition is met
                break;
            }
        }

        // If regression_b_half is still equal to the initial value, it means no entry met the condition in the loop.
        // Then it panics and stops the program.
        if regression_b_half == 123456789.12 {
            panic!("ILLEGAL");
        }

        regression_dataset.push(
            RegressionData {
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

// Function to construct the basic regression dataset
fn make_basic_regression_dataset(price_dataset: Vec<PriceData>, begin_timestamp: Option<f64>) -> Vec<RegressionDataTemporary> {
    let mut result: Vec<RegressionDataTemporary> = Vec::new();
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
        result.push(RegressionDataTemporary {
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

// Functions to calculate individual regression parameters
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