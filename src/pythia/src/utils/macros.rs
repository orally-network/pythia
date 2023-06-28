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
macro_rules! retry_until_success {
    ($func:expr) => {{
        let mut attempts = 1;
        let mut result = $func.await;

        while result.is_err()
            && format!("{:?}", result.as_ref().unwrap_err())
                .contains("Canister http responses were different across replicas")
        {
            result = $func.await;
            attempts += 1;
        }

        let (func_name, _) = stringify!($func).rsplit_once("(").unwrap();

        ic_utils::logger::log_message(format!("[{func_name}] used attempts: {attempts}"));

        result
    }};
}
