use derive_builder::Builder;
use ic_web3_rs::transforms::transform::TransformProcessor;
use serde_json::Value;

#[derive(Debug, Builder, Default)]
pub struct RawTxExecutionTransformProcessor {
    pub transaction_index: bool,
    pub log_index: bool,
}

impl TransformProcessor for RawTxExecutionTransformProcessor {
    fn process_body(&self, body: &[u8]) -> Vec<u8> {
        let mut body: Value = serde_json::from_slice(body).expect("Should be valid json");

        let result = body
            .get_mut("result")
            .expect("Should have result field")
            .as_array_mut();
        if result.is_none() {
            return serde_json::to_vec(&body).expect("Should be valid json");
        }

        let elements = result.expect("Should be valid json");
        for element in elements.iter_mut() {
            if self.transaction_index {
                element
                    .as_object_mut()
                    .expect("Should be valid json")
                    .insert("transactionIndex".to_string(), Value::from("0x0"));
            }
            if self.log_index {
                element
                    .as_object_mut()
                    .expect("Should be valid json")
                    .insert("logIndex".to_string(), Value::from("0x0"));
            }
        }
        serde_json::to_vec(&body).expect("Should be valid json")
    }
}

pub fn raw_tx_execution_transform_processor() -> RawTxExecutionTransformProcessor {
    RawTxExecutionTransformProcessorBuilder::default()
        .log_index(true)
        .transaction_index(true)
        .build()
        .expect("Should be valid builder")
}
