#[macro_export]
macro_rules! block_on {
    ( $rules:expr ) => {{
        let mut res;

        cfg_if::cfg_if! {
            if #[cfg(target_arch = "wasm32")] {
                wasm_bindgen_futures::spawn_local(async move {
                    res = $rules.await.unwrap();
                })
            } else {
                pollster::block_on(async {
                    res = $rules.await.unwrap();
                });
            }
        }

        res
    };};
}
