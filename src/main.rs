use chrono::{Datelike, Timelike, Utc};
use chrono_tz::Asia::Taipei;
use csv::{Reader, Writer};
use serde_json::Value;
use std::fs::File;

const SIX_HOURS_IN_SECONDS: u32 = 6 * 3600;
const TEN_MINUTES_IN_SECONDS: u32 = 10 * 60;
const NUM_DATA_PER_DAY: u32 = 97;

#[tokio::main]
async fn main() {
    let res = Fetcher::fetch().await;
    let i = NUM_DATA_PER_DAY * (res.weekday - 1)
        + (res.time - SIX_HOURS_IN_SECONDS) / TEN_MINUTES_IN_SECONDS;

    let file_path = "data.csv";
    let mut data = Data::from(file_path);
    let record = data.records.get_mut(i as usize).unwrap();
    record.count += res.count;
    record.sum += res.sum;

    data.write(file_path);
}

#[derive(Debug)]
struct Record {
    weekday: u32,
    time: u32,
    count: u32,
    sum: u32,
}

struct Data {
    records: Vec<Record>,
}

impl Data {
    pub fn from(file_path: &str) -> Data {
        let file = File::open(file_path).unwrap();
        let mut reader = Reader::from_reader(file);
        let data: Vec<Record> = reader
            .records()
            .map(|r| {
                let r = r.unwrap();
                Record {
                    weekday: r.get(0).unwrap().parse().unwrap(),
                    time: r.get(1).unwrap().parse().unwrap(),
                    count: r.get(2).unwrap().parse().unwrap(),
                    sum: r.get(3).unwrap().parse().unwrap(),
                }
            })
            .collect();

        Data { records: data }
    }

    pub fn write(&self, file_path: &str) {
        let mut writer = Writer::from_path(file_path).unwrap();
        writer.write_record(&["weekday", "time", "count", "sum", "average"]).unwrap();
        for r in self.records.iter() {
            let average = if let 0 = r.count { 0 } else { r.sum / r.count };
            writer
                .write_record(&[
                    r.weekday.to_string(),
                    r.time.to_string(),
                    r.count.to_string(),
                    r.sum.to_string(),
                    average.to_string(),
                ])
                .unwrap();
        }
    }
}

struct Fetcher {}

impl Fetcher {
    pub async fn fetch() -> Record {
        let url = "https://dasc.cyc.org.tw/api";
        let res = reqwest::Client::new()
            .post(url)
            .send()
            .await
            .unwrap()
            .text()
            .await
            .unwrap();

        let t = Utc::now().with_timezone(&Taipei);

        let data: Value = serde_json::from_str(&res).unwrap();
        let s = &data["swim"][0].to_string();
        let v = s[1..s.len() - 1].parse().unwrap();

        Record {
            weekday: t.weekday().number_from_monday(),
            time: t.num_seconds_from_midnight(),
            count: 1,
            sum: v,
        }
    }
}
