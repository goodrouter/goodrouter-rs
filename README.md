# Goodrouter, the rust edition

A good router should:

- [x] work in a server or a client (or any other) environment
- [x] be able to construct routes based on their name
- [x] should have a simple API!
- [x] not do the actual navigation!
- [x] be framework agnostic
- [x] be very minimal and simple!

Check out our (website)[https://www.goodrouter.org], join our (Discord server)[https://discord.gg/BJ8v7xTq8d]!

## Example

```rust
let mut router = Router::new();

router.insert_route("all-products", "/product/all");
router.insert_route("product-detail", "/product/{id}");

// And now we can parse routes!

{
  let route = router.parse_route("/not-found");
  assert_eq!(route, None);
}

{
  let route = router.parse_route("/product/all");
  assert_eq!(route, Some(Route{
    name: "all-products".to_owned(),
    parameters: vec![],
  }));
}

{
  let route = router.parse_route("/product/1");
  assert_eq!(route, Some(Route{
    name: "product-detail".to_owned(),
    parameters: vec![
      ("id", "1"),
    ],
  }));
}

// And we can stringify routes

{
  let path = router.stringify_route(Route{
    name: "all-products".to_owned(),
        parameters: vec![],
  });
  assert_eq!(path, "/product/all".to_owned(),);
}

{
  let path = router.stringify_route(Route{
    name: "product-detail".to_owned(),
    parameters: vec![
      ("id", "1"),
    ],
  });
  assert_eq!(path, "/product/2".to_owned());
}
```
