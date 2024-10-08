use super::*;
use salvo::Router;

pub fn init_router() -> Router {
  let mut router = Router::new();
  let sub_routers: Vec<fn() -> Router> =
    vec![user::init_router, problem::init_router, record::init_router];
  for sub_router in sub_routers {
    router = router.push(sub_router());
  }
  return router;
}
