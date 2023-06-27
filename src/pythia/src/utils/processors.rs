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
        let mut body: Value = serde_json::from_slice(body).unwrap();
        
        let result = body
            .get_mut("result")
            .unwrap()
            .as_array_mut();
        if result.is_none() {
            return serde_json::to_vec(&body).unwrap();
        }
    
        let elements = result
            .unwrap();
        for element in elements.iter_mut() {
            if self.transaction_index {
                element
                    .as_object_mut()
                    .unwrap()
                    .insert("transactionIndex".to_string(), Value::from("0x0"));
            }
            if self.log_index {
                element
                    .as_object_mut()
                    .unwrap()
                    .insert("logIndex".to_string(), Value::from("0x0"));
            }
        }
        serde_json::to_vec(&body).unwrap()
    }
}

pub fn raw_tx_execution_transform_processor() -> RawTxExecutionTransformProcessor {
    RawTxExecutionTransformProcessorBuilder::default()
        .log_index(true)
        .transaction_index(true)
        .build()
        .unwrap()
}
