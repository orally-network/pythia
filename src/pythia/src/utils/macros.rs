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
        let mut attempts = 1;
        let mut result = $func.await;

        while result.is_err()
            && (format!("{:?}", result.as_ref().unwrap_err()).contains("Canister http responses were different across replicas")
            || format!("{:?}", result.as_ref().unwrap_err()).contains("Timeout expired")
            || format!("{:?}", result.as_ref().unwrap_err()).contains("pending")) // or Exchange rate canister error: pending
            && attempts <= 5
        {
            result = $func.await;
            attempts += 1;
        }

        let (func_name, func_other) = stringify!($func).rsplit_once("(").unwrap();

        ic_utils::logger::log_message(format!("[{func_name} : {func_other}] used attempts: {attempts}"));

        result
    }};
}
