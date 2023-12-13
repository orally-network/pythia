use std::{cell::RefCell, collections::HashMap, io};

use candid::CandidType;
use serde::{Deserialize, Serialize};

#[derive(CandidType, Clone, Debug, Default, Deserialize, Serialize)]
pub struct Metric {
    pub name: String,
    pub help: String,
    pub typ: String,
    pub label_names: Vec<String>,
    pub inner: HashMap<Vec<String>, Vec<Inner>>,
}

#[derive(CandidType, Clone, Debug, Default, Deserialize, Serialize)]
pub struct Inner {
    pub value: u128,
    pub label_values: Vec<String>,
}

impl Inner {
    pub fn new(value: u128, label_values: Vec<String>) -> Self {
        Self {
            value,
            label_values,
        }
    }

    pub fn inc(&mut self) {
        self.value = self.value.saturating_add(1);
    }

    pub fn dec(&mut self) {
        self.value = self.value.saturating_sub(1);
    }

    pub fn set(&mut self, value: u128) {
        self.value = value;
    }

    pub fn get(&self) -> u128 {
        self.value
    }
}

#[allow(unused)]
impl Metric {
    pub fn new(name: &str, help: &str, typ: &str, label_names: &[&str]) -> Self {
        let label_names = label_names.iter().map(|s| s.to_string()).collect();
        Self {
            name: name.to_string(),
            help: help.to_string(),
            typ: typ.to_string(),
            label_names,
            inner: HashMap::new(),
        }
    }

    pub fn inc(&mut self) {
        self.inner
            .entry(vec![])
            .or_insert_with(|| vec![Inner::new(1, vec![])]);
    }

    pub fn dec(&mut self) {
        self.inner
            .entry(vec![])
            .or_insert_with(|| vec![Inner::new(0, vec![])]);
    }

    pub fn set(&mut self, value: u128) {
        self.inner
            .entry(vec![])
            .or_insert_with(|| vec![Inner::new(value, vec![])]);
    }

    pub fn get(&self) -> u128 {
        self.inner
            .get(&vec![])
            .map(|inner_vec| inner_vec.first().unwrap().value)
            .unwrap_or(0)
    }

    fn check_label_values(&self, label_values: &[String]) {
        if label_values.len() != self.label_names.len() {
            panic!(
                "Invalid number of labels. Expected {}, got {}",
                self.label_names.len(),
                label_values.len()
            )
        }
    }

    pub fn with_label_values(&mut self, label_values: Vec<String>) -> &mut Inner {
        self.check_label_values(&label_values);

        self.inner
            .entry(label_values.clone())
            .or_insert_with(|| vec![Inner::new(0, label_values)])
            .first_mut()
            .unwrap()
    }

    fn encode_header<W: io::Write>(&self, w: &mut W) -> io::Result<()> {
        writeln!(w, "# HELP {} {}", self.name, self.help)?;
        writeln!(w, "# TYPE {} {}", self.name, self.typ)
    }

    fn encode_value<W: io::Write>(&self, w: &mut W) -> io::Result<()> {
        for (label_values, inner_vec) in self.inner.iter() {
            if label_values.is_empty() {
                writeln!(w, "{} {}", self.name, inner_vec.first().unwrap().value)?;
                return Ok(());
            }

            for inner in inner_vec {
                let labels = self
                    .label_names
                    .iter()
                    .zip(label_values.iter())
                    .map(|(label_name, label_value)| format!("{}=\"{}\"", label_name, label_value))
                    .collect::<Vec<String>>()
                    .join(",");

                writeln!(w, "{}{{{}}} {}", self.name, labels, inner.value)?;
            }
        }

        Ok(())
    }

    pub fn encode<W: io::Write>(&self, w: &mut W) -> std::io::Result<()> {
        if self.inner.is_empty() {
            return Ok(());
        }

        self.encode_header(w)?;
        self.encode_value(w)
    }
}

#[allow(non_snake_case)]
#[derive(CandidType, Clone, Debug, Default, Deserialize, Serialize)]
pub struct Metrics {
    pub ACTIVE_SUBSCRIPTIONS: Metric,
    pub RPC_OUTCALLS: Metric,
    pub SUCCESSFUL_RPC_OUTCALLS: Metric,
    pub SYBIL_OUTCALLS: Metric,
    pub SUCCESSFUL_SYBIL_OUTCALLS: Metric,
    pub CYCLES: Metric,
}

impl Metrics {
    pub fn encode<W: io::Write>(&self, w: &mut W) -> std::io::Result<()> {
        self.ACTIVE_SUBSCRIPTIONS.encode(w)?;
        self.RPC_OUTCALLS.encode(w)?;
        self.SUCCESSFUL_RPC_OUTCALLS.encode(w)?;
        self.SYBIL_OUTCALLS.encode(w)?;
        self.SUCCESSFUL_SYBIL_OUTCALLS.encode(w)?;
        self.CYCLES.encode(w)
    }
}

thread_local! {
    pub static METRICS: RefCell<Metrics> = RefCell::new(Metrics{
            ACTIVE_SUBSCRIPTIONS: Metric::new(
                "active_subscriptions",
                "Number of active subscriptions",
                "gauge",
                &["chain"],
            ),

            RPC_OUTCALLS: Metric::new(
                "rpc_outcalls",
                "Number of rpc outcalls",
                "counter",
                &["method"],
            ),

            SUCCESSFUL_RPC_OUTCALLS: Metric::new(
                "sucessfull_rpc_outcalls",
                "Number of successfully returned rpc outcalls",
                "counter",
                &["method"],
            ),

            SYBIL_OUTCALLS: Metric::new(
                "sybil_outcalls",
                "Number of sybil outcalls",
                "counter",
                &["method"],
            ),

            SUCCESSFUL_SYBIL_OUTCALLS: Metric::new(
                "successful_sybil_outcalls",
                "Number of successfully returned sybil outcalls. Note that this metric is about ic communication with the sybil canister. Meaning if sybil returns error, this metric will not be incremented. If ic returns error while quering sybil canister, this metric will be incremented.",
                "counter",
                &["method"],
            ),

            CYCLES: Metric::new(
                "cycles",
                "Number of canister's cycles",
                "gauge",
                &[],
            )
    });
    // pub static PYTHIA_REGISTRY: RefCell<Registry> = RefCell::new(Registry::new_custom(
    //     Some("slurp_parser".to_string()),
    //     Some(HashMap::from([("version".to_string(), env!("CARGO_PKG_VERSION").to_string())])),
    // )
    // .unwrap());

            // pub static ACTIVE_SUBSCRIPTIONS: IntGaugeVec = register_int_gauge_vec!(
            //     "active_subscriptions",
            //     "Number of active subscriptions",
            //     &["chain"],
            // ).unwrap();

            // pub static RPC_OUTCALLS: IntCounterVec = register_int_counter_vec!(
            //     "rpc_outcalls",
            //     "Number of rpc outcalls",
            //     &["method"],
            // ).unwrap();

            // pub static SUCCESSFUL_RPC_OUTCALLS: IntCounterVec = register_int_counter_vec!(
            //     "sucessfull_rpc_outcalls",
            //     "Number of successfully returned rpc outcalls",
            //     &["method"],
            // ).unwrap();

            // pub static SYBIL_OUTCALLS: IntCounterVec = register_int_counter_vec!(
            //     "sybil_outcalls",
            //     "Number of sybil outcalls",
            //     &["method"],
            // ).unwrap();

            // pub static SUCCESSFUL_SYBIL_OUTCALLS: IntCounterVec = register_int_counter_vec!(
            //     "successful_sybil_outcalls",
            //     "Number of successfully returned sybil outcalls. Note that this metric is about ic communication with the sybil canister. Meaning if sybil returns error, this metric will not be incremented. If ic returns error while quering sybil canister, this metric will be incremented.",
            //     &["method"],
            // ).unwrap();

            // pub static CYCLES: IntGauge = register_int_gauge!(
            //     "cycles",
            //     "Number of canister's cycles",
            // ).unwrap();
}

pub fn gather_metrics() -> Vec<u8> {
    // let encoder = TextEncoder::new();

    // // let metric_families = PYTHIA_REGISTRY.with(|r| r.borrow().gather());
    // let metric_families = prometheus::gather();
    let mut buffer = vec![];
    // encoder.encode(&metric_families, &mut buffer).unwrap();

    METRICS.with(|m| m.borrow().encode(&mut buffer).unwrap());

    buffer
}

#[macro_export]
macro_rules! metrics {
    ( inc $metric:ident ) => {
        $crate::utils::metrics::METRICS.with(|m| m.borrow_mut().$metric.inc());
    };

    ( inc $metric:ident, $($labels:expr),+) => {{
        let lbls: Vec<String> = vec![$(format!("{}", $labels)),+];

        $crate::utils::metrics::METRICS.with(|m| m.borrow_mut().$metric.with_label_values(lbls).inc());
        $crate::utils::metrics::METRICS.with(|m| m.borrow_mut().$metric.with_label_values(vec!["all".to_string()]).inc());
    }};

    ( dec $metric:ident ) => {
        $crate::utils::metrics::METRICS.with(|m| m.borrow_mut_mut().$metric.dec());
    };

    ( dec $metric:ident, $($labels:expr),+) => {
        let lbls: Vec<String> = vec![$(format!("{}", $labels)),+];

        $crate::utils::metrics::METRICS.with(|m| m.borrow_mut().$metric.with_label_values(lbls).dec());
        $crate::utils::metrics::METRICS.with(|m| m.borrow_mut().$metric.with_label_values(vec!["all".to_string()]).dec());
    };

    ( get $metric:ident ) => {
        $crate::utils::metrics::METRICS.with(|m| m.borrow_mut().$metric.get())
    };

    ( get $metric:ident, $($labels:expr),+) => {
        {
            let lbls: Vec<String> = vec![$(format!("{}", $labels)),+];

            $crate::utils::metrics::METRICS.with(|m| m.borrow_mut().$metric.with_label_values(lbls).get())
        }
    };

    ( timer $metric:ident, $($labels:expr),+) => {
        let lbls: Vec<String> = vec![$(format!("{}", $labels)),+];

        $crate::utils::metrics::METRICS.with(|m| m.borrow_mut().$metric.with_label_values(lbls).start_timer());
    };

    ( timer $metric:ident) => {
        $crate::utils::metrics::METRICS.with(|m| m.borrow_mut().$metric.start_timer())
    };

    ( timer observe $timer:ident) => {
        $timer.observe_duration()
    };

    ( timer discard $timer:ident) => {
        $timer.stop_and_discard()
    };

    ( set $metric:ident, $val:expr ) => {
        $crate::utils::metrics::METRICS.with(|m| m.borrow_mut().$metric.set($val));
    };

    ( set $metric:ident, $val:expr, $($labels:expr),+) => {
        let lbls: Vec<String> = vec![$(format!("{}", $labels)),+];

        $crate::utils::metrics::METRICS.with(|m| m.borrow_mut().$metric.with_label_values(lbls).set($val));
        $crate::utils::metrics::METRICS.with(|m| m.borrow_mut().$metric.with_label_values(vec!["all".to_string()]).set($val));
    };
}
