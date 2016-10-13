use server::NewHandler;


impl<T, H> NewHandler for T
    where T: Fn() -> H
{
    type Handler = H;
    fn new_handler(&self) -> H {
        self()
    }
}
