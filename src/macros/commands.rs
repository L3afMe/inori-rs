#[macro_export]
macro_rules! parse_arg {
    ($ctx:expr, $msg:expr, $args:expr, $arg_name:literal, $arg_type:ty) => {
        if let Ok(val) = $args.single::<$arg_type>() {
            val
        } else {
            return $msg
                .channel_id
                .send_tmp($ctx, |m: &mut MessageCreator| {
                    m.error().title("Error").content(format!("Unable to parse {}", $arg_name))
                })
                .await;
        }
    };
}
