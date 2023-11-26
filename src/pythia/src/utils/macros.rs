#[macro_export]
macro_rules! clone_with_state {
    ($field:ident) => {{
        $crate::STATE.with(|state| state.borrow().$field.clone())
    }};
}

#[macro_export]
macro_rules! update_state {
    ($field:ident, $value:expr) => {{
        $crate::STATE.with(|state| {
            state.borrow_mut().$field = $value;
        })
    }};
}

#[macro_export]
macro_rules! log {
    ($($arg:tt)*) => {{
        ic_cdk::println!($($arg)*);
        ic_utils::logger::log_message(format!($($arg)*));
        ic_utils::monitor::collect_metrics();
    }};
}

#[macro_export]
macro_rules! dig {
    ($state:ident, $field:ident, $chain_id:ident, $key:ident) => {
        $state
            .$field
            .0
            .get($chain_id)
            .context($crate::types::errors::PythiaError::ChainDoesNotExist)?
            .get($key)
    };
}

#[macro_export]
macro_rules! dig_mut {
    ($state:ident, $field:ident, $chain_id:ident, $key:ident) => {
        $state
            .$field
            .0
            .get_mut($chain_id)
            .context($crate::types::errors::PythiaError::ChainDoesNotExist)?
            .get_mut($key)
    };
}

#[macro_export]
macro_rules! retry_until_success {
    ($func:expr) => {{
        const MAX_RETRIES: u32 = 10;
        const DURATION_BETWEEN_ATTEMPTS: std::time::Duration = std::time::Duration::from_millis(5000);

        let mut attempts = 0u32;
        let mut result = $func.await;

        let (func_name, func_other) = stringify!($func).rsplit_once("(").unwrap();

        while result.is_err()
            && (format!("{:?}", result.as_ref().unwrap_err()).contains("Canister http responses were different across replicas")
            || format!("{:?}", result.as_ref().unwrap_err()).contains("Timeout expired")
            || format!("{:?}", result.as_ref().unwrap_err()).contains("SysTransient")
            || format!("{:?}", result.as_ref().unwrap_err()).contains("pending")) // or Exchange rate canister error: pending
            && attempts < MAX_RETRIES
        {
            crate::utils::sleep(DURATION_BETWEEN_ATTEMPTS).await;
            result = $func.await;
            ic_utils::logger::log_message(format!("[{func_name} : {func_other}] attempt: {attempts}"));
            attempts += 1;
        }



        result
    }};
}
