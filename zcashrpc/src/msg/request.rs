pub trait Request {
    type Response;

    fn name() -> &'static str;
}
