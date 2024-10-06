
use arrow::array::{Float64Array, StringArray};
use arrow::csv::ReaderBuilder;
use arrow::record_batch::RecordBatch;
use trusty::Trees;
use criterion::{criterion_group, criterion_main, Criterion};
use tokio::runtime::Runtime;
use std::fs::File;
use std::io::BufReader;
use std::sync::Arc;
use arrow::datatypes::{DataType, Field, Schema};
use serde_json::Value;

use std::error::Error;

async fn run_prediction(trees:&Trees, batches: &[RecordBatch]) -> Result<(), Box<dyn std::error::Error>> {
    for batch in batches {
        let _prediction = trees.predict_batch(batch);
    }
    Ok(())
}

fn read_csv_to_batches(path: &str, batch_size: usize) -> Result<Vec<RecordBatch>, Box<dyn Error>> {
    let file = File::open(path)?;

    let schema = Arc::new(Schema::new(vec![
        Field::new("carat", DataType::Float64, false),
        Field::new("cut", DataType::Utf8, false),
        Field::new("color", DataType::Utf8, false),
        Field::new("clarity", DataType::Utf8, false),
        Field::new("depth", DataType::Float64, false),
        Field::new("table", DataType::Float64, false),
        Field::new("price", DataType::Int64, false),
        Field::new("x", DataType::Float64, false),
        Field::new("y", DataType::Float64, false),
        Field::new("z", DataType::Float64, false),
    ]));

    let mut csv = ReaderBuilder::new(schema)
        .has_header(true)
        .with_batch_size(batch_size)
        .build(file)?;

    let mut batches = Vec::new();
    while let Some(batch) = csv.next() {
        batches.push(batch?);
    }

    Ok(batches)
}

fn preprocess_batches(batches: &[RecordBatch]) -> Result<Vec<RecordBatch>, Box<dyn Error>> {
    let schema = Arc::new(Schema::new(vec![
        Field::new("carat", DataType::Float64, false),
        Field::new("depth", DataType::Float64, false),
        Field::new("table", DataType::Float64, false),
        Field::new("x", DataType::Float64, false),
        Field::new("y", DataType::Float64, false),
        Field::new("z", DataType::Float64, false),
        Field::new("cut_good", DataType::Float64, false),
        Field::new("cut_ideal", DataType::Float64, false),
        Field::new("cut_premium", DataType::Float64, false),
        Field::new("cut_very_good", DataType::Float64, false),
        Field::new("color_e", DataType::Float64, false),
        Field::new("color_f", DataType::Float64, false),
        Field::new("color_g", DataType::Float64, false),
        Field::new("color_h", DataType::Float64, false),
        Field::new("color_i", DataType::Float64, false),
        Field::new("color_j", DataType::Float64, false),
        Field::new("clarity_if", DataType::Float64, false),
        Field::new("clarity_si1", DataType::Float64, false),
        Field::new("clarity_si2", DataType::Float64, false),
        Field::new("clarity_vs1", DataType::Float64, false),
        Field::new("clarity_vs2", DataType::Float64, false),
        Field::new("clarity_vvs1", DataType::Float64, false),
        Field::new("clarity_vvs2", DataType::Float64, false),
    ]));

    let mut processed_batches = Vec::new();

    for batch in batches {
        let carat = batch.column(0).as_any().downcast_ref::<Float64Array>().unwrap();
        let cut = batch.column(1).as_any().downcast_ref::<StringArray>().unwrap();
        let color = batch.column(2).as_any().downcast_ref::<StringArray>().unwrap();
        let clarity = batch.column(3).as_any().downcast_ref::<StringArray>().unwrap();
        let depth = batch.column(4).as_any().downcast_ref::<Float64Array>().unwrap();
        let table = batch.column(5).as_any().downcast_ref::<Float64Array>().unwrap();
        let x = batch.column(7).as_any().downcast_ref::<Float64Array>().unwrap();
        let y = batch.column(8).as_any().downcast_ref::<Float64Array>().unwrap();
        let z = batch.column(9).as_any().downcast_ref::<Float64Array>().unwrap();

        let row_count = batch.num_rows();

        let mut cut_good = vec![0.0; row_count];
        let mut cut_ideal = vec![0.0; row_count];
        let mut cut_premium = vec![0.0; row_count];
        let mut cut_very_good = vec![0.0; row_count];

        let mut color_e = vec![0.0; row_count];
        let mut color_f = vec![0.0; row_count];
        let mut color_g = vec![0.0; row_count];
        let mut color_h = vec![0.0; row_count];
        let mut color_i = vec![0.0; row_count];
        let mut color_j = vec![0.0; row_count];

        let mut clarity_if = vec![0.0; row_count];
        let mut clarity_si1 = vec![0.0; row_count];
        let mut clarity_si2 = vec![0.0; row_count];
        let mut clarity_vs1 = vec![0.0; row_count];
        let mut clarity_vs2 = vec![0.0; row_count];
        let mut clarity_vvs1 = vec![0.0; row_count];
        let mut clarity_vvs2 = vec![0.0; row_count];

        for i in 0..row_count {
            match cut.value(i) {
                "Good" => cut_good[i] = 1.0,
                "Ideal" => cut_ideal[i] = 1.0,
                "Premium" => cut_premium[i] = 1.0,
                "Very Good" => cut_very_good[i] = 1.0,
                _ => {}
            }

            match color.value(i) {
                "E" => color_e[i] = 1.0,
                "F" => color_f[i] = 1.0,
                "G" => color_g[i] = 1.0,
                "H" => color_h[i] = 1.0,
                "I" => color_i[i] = 1.0,
                "J" => color_j[i] = 1.0,
                _ => {}
            }

            match clarity.value(i) {
                "IF" => clarity_if[i] = 1.0,
                "SI1" => clarity_si1[i] = 1.0,
                "SI2" => clarity_si2[i] = 1.0,
                "VS1" => clarity_vs1[i] = 1.0,
                "VS2" => clarity_vs2[i] = 1.0,
                "VVS1" => clarity_vvs1[i] = 1.0,
                "VVS2" => clarity_vvs2[i] = 1.0,
                _ => {}
            }
        }

        let processed_batch = RecordBatch::try_new(
            schema.clone(),
            vec![
                Arc::new(carat.clone()),
                Arc::new(depth.clone()),
                Arc::new(table.clone()),
                Arc::new(x.clone()),
                Arc::new(y.clone()),
                Arc::new(z.clone()),
                Arc::new(Float64Array::from(cut_good)),
                Arc::new(Float64Array::from(cut_ideal)),
                Arc::new(Float64Array::from(cut_premium)),
                Arc::new(Float64Array::from(cut_very_good)),
                Arc::new(Float64Array::from(color_e)),
                Arc::new(Float64Array::from(color_f)),
                Arc::new(Float64Array::from(color_g)),
                Arc::new(Float64Array::from(color_h)),
                Arc::new(Float64Array::from(color_i)),
                Arc::new(Float64Array::from(color_j)),
                Arc::new(Float64Array::from(clarity_if)),
                Arc::new(Float64Array::from(clarity_si1)),
                Arc::new(Float64Array::from(clarity_si2)),
                Arc::new(Float64Array::from(clarity_vs1)),
                Arc::new(Float64Array::from(clarity_vs2)),
                Arc::new(Float64Array::from(clarity_vvs1)),
                Arc::new(Float64Array::from(clarity_vvs2)),
            ],
        )?;

        processed_batches.push(processed_batch);
    }

    Ok(processed_batches)
}

fn bench_trusty(c: &mut Criterion) -> Result<(), Box<dyn Error>> {
    let rt = Runtime::new()?;
    
    // Load the model data once, outside the benchmark loop
    let model_file = File::open("models/pricing-model-100-mod.json")
        .or_else(|_| File::open("../models/pricing-model-100-mod.json"))
        .map_err(|e| format!("Failed to open model file: {}", e))?;
    
    let reader = BufReader::new(model_file);
    let model_data: Value = serde_json::from_reader(reader)
        .map_err(|e| format!("Failed to parse JSON: {}", e))?;
    
    let trees = Trees::load(&model_data);

    // Read and preprocess the CSV data
    let raw_batches = read_csv_to_batches("diamonds.csv", 8000)?;
    let batches = preprocess_batches(&raw_batches)?;

    c.bench_function("trusty", |b| {
        b.to_async(&rt)
            .iter(|| async { 
                run_prediction(&trees, &batches).await.unwrap() })
    });

    Ok(())
}

criterion_group!(benches, bench_trusty);
criterion_main!(benches);