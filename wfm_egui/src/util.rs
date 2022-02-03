pub fn clone_option_with_inner_ref<T: Clone>(v: Option<&T>) -> Option<T> {
    v.map(|v| v.clone())
}
