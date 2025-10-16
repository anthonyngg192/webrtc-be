use crate::utils::APP_NAME;
use mediasoup::prelude::{Consumer, ConsumerId, MediaKind, Producer, ProducerId};
use once_cell::sync::Lazy;
use prometheus::{
    register_counter_vec, register_histogram_vec, CounterVec, HistogramVec, Opts, Registry,
};
use std::sync::Arc;

pub static REGISTRY: Lazy<Registry> =
    Lazy::new(|| Registry::new_custom(Some(APP_NAME.to_string()), None).unwrap());

/// ================= Producer Metrics =====================
static PRODUCER_BANDWIDTH: Lazy<CounterVec> = Lazy::new(|| {
    register_counter_vec!(
        Opts::new("producer_bandwidth_bytes_total", "Producer bandwidth"),
        &["room_id", "producer_id", "kind"]
    )
    .unwrap()
});

static PRODUCER_PACKET_LOSS: Lazy<CounterVec> = Lazy::new(|| {
    register_counter_vec!(
        Opts::new("producer_packets_lost_total", "Producer packet lost"),
        &["room_id", "producer_id", "kind"]
    )
    .unwrap()
});

static PRODUCER_LATENCY: Lazy<HistogramVec> = Lazy::new(|| {
    register_histogram_vec!(
        "producer_latency_seconds",
        "Producer latency",
        &["room_id", "producer_id", "kind"],
        vec![0.01, 0.05, 0.1, 0.2, 0.5, 1.0]
    )
    .unwrap()
});

/// ================= Consumer Metrics =====================
static CONSUMER_BANDWIDTH: Lazy<CounterVec> = Lazy::new(|| {
    register_counter_vec!(
        Opts::new("consumer_bandwidth_bytes_total", "Consumer bandwidth"),
        &["room_id", "producer_id", "consumer_id"]
    )
    .unwrap()
});

static CONSUMER_PACKET_LOSS: Lazy<CounterVec> = Lazy::new(|| {
    register_counter_vec!(
        Opts::new("consumer_packets_lost_total", "Consumer packet lost"),
        &["room_id", "producer_id", "consumer_id"]
    )
    .unwrap()
});

static CONSUMER_LATENCY: Lazy<HistogramVec> = Lazy::new(|| {
    register_histogram_vec!(
        "consumer_latency_seconds",
        "Consumer latency",
        &["room_id", "producer_id", "consumer_id"],
        vec![0.01, 0.05, 0.1, 0.2, 0.5, 1.0]
    )
    .unwrap()
});

pub async fn process_producer_metrics(room_code: &str, producers: Vec<Arc<Producer>>) {
    for producer in producers {
        let stats = producer.clone().get_stats().await.expect("producer error");
        let mut total_bytes = 0u64;
        let mut total_lost = 0u64;
        let mut latency_sum = 0.0;
        let mut count = 0;

        for stat in stats {
            total_bytes += stat.byte_count;
            total_lost += stat.packets_lost;
            if let Some(rtt) = stat.round_trip_time {
                latency_sum += rtt as f64;
                count += 1;
            }
        }

        let avg_latency = if count > 0 {
            latency_sum / count as f64
        } else {
            0.0
        };

        let kind = producer.kind();
        update_producer_metrics(
            room_code,
            &producer.id(),
            total_bytes,
            total_lost,
            avg_latency,
            &kind,
        );
    }
}

pub async fn process_consumer_metrics(room_code: &str, consumers: Vec<Arc<Consumer>>) {
    for consumer in consumers {
        let stats_consumer = consumer.get_stats().await.expect("Error incase  get stats");

        let stat = stats_consumer.consumer_stats();

        update_consumer_metrics(
            room_code,
            &consumer.producer_id(),
            &consumer.id(),
            stat.byte_count,
            stat.packet_count,
            stat.round_trip_time.unwrap_or(0.0) as f64,
        );
    }
}

fn update_producer_metrics(
    room_code: &str,
    producer_id: &ProducerId,
    total_bytes: u64,
    total_lost: u64,
    avg_rtt: f64,
    media_kind: &MediaKind,
) {
    let producer_id = producer_id.to_string();
    let kind = match media_kind {
        MediaKind::Audio => "Audio",
        MediaKind::Video => "Video",
    };
    let labels = &[room_code, producer_id.as_str(), kind];

    PRODUCER_BANDWIDTH
        .with_label_values(labels)
        .inc_by(total_bytes as f64);

    PRODUCER_PACKET_LOSS
        .with_label_values(labels)
        .inc_by(total_lost as f64);

    if avg_rtt > 0.0 {
        PRODUCER_LATENCY.with_label_values(labels).observe(avg_rtt);
    }
}

pub fn update_consumer_metrics(
    room_id: &str,
    producer_id: &ProducerId,
    consumer_id: &ConsumerId,
    total_bytes: u64,
    total_lost: u64,
    avg_rtt: f64,
) {
    let labels = &[room_id, &producer_id.to_string(), &consumer_id.to_string()];

    CONSUMER_BANDWIDTH
        .with_label_values(labels)
        .inc_by(total_bytes as f64);

    CONSUMER_PACKET_LOSS
        .with_label_values(labels)
        .inc_by(total_lost as f64);

    if avg_rtt > 0.0 {
        CONSUMER_LATENCY.with_label_values(labels).observe(avg_rtt);
    }
}

pub fn remove_label_producer(
    room_code: &str,
    producer_id: &ProducerId,
    kind: &MediaKind,
    consumer_ids: Vec<Arc<ConsumerId>>,
) {
    let producer_id = producer_id.to_string();
    let kind = match kind {
        MediaKind::Audio => "Audio",
        MediaKind::Video => "Video",
    };
    let labels = &[room_code, producer_id.as_str(), kind];

    let _ = PRODUCER_BANDWIDTH.remove_label_values(labels);
    let _ = PRODUCER_LATENCY.remove_label_values(labels);
    let _ = PRODUCER_PACKET_LOSS.remove_label_values(labels);

    for consumer_id in consumer_ids {
        remove_label_consumer_only(room_code, producer_id.as_str(), &consumer_id);
    }
}

pub fn remove_label_consumer_only(room_code: &str, producer_id: &str, consumer_id: &ConsumerId) {
    let labels = &[room_code, producer_id, &consumer_id.to_string()];

    let _ = CONSUMER_BANDWIDTH.remove_label_values(labels);
    let _ = CONSUMER_LATENCY.remove_label_values(labels);
    let _ = CONSUMER_PACKET_LOSS.remove_label_values(labels);
}

pub fn remove_label_producer_only(room_code: &str, producer_id: &ProducerId, kind: &MediaKind) {
    let producer_id = producer_id.to_string();
    let kind = match kind {
        MediaKind::Audio => "Audio",
        MediaKind::Video => "Video",
    };
    let labels = &[room_code, producer_id.as_str(), kind];

    let _ = PRODUCER_BANDWIDTH.remove_label_values(labels);
    let _ = PRODUCER_LATENCY.remove_label_values(labels);
    let _ = PRODUCER_PACKET_LOSS.remove_label_values(labels);
}
