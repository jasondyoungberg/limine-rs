# 0.1.4
* Adds an optional feature (`requests-section`) which brings in the `#[limine_tag]` macro. This macro is used to
  insert the limine request in the `.limine_reqs` section. Checkout the Limine Specification's Limine Requests 
  Section for more information.

  ## Example
  ```rust
  #[limine_tag]
  static BOOTLOADER_INFO: LimineBootInfoRequest = LimineBootInfoRequest::new(0);
  ```

# 0.1.3
* Fix the broken layout of the `LimineTerminal` structure.
* Make use of NPO ([Null Pointer Optimization](https://doc.rust-lang.org/std/option/index.html#representation)) inside the `LiminePtr` structure for safety and to be more explicit.

# 0.1.2
* **Breaking**: The `write` function now takes a `&LimineTerminal` as an argument as expected. In addition to that, the
                `write` function returns an `Option` containg the writer helper closure function since a faulty bootloader *can*
                return null terminal write function pointer.

# 0.1.1
* **Breaking**: The `response` field for the request structures is now private and the `get_response` function must be used instead to retrieve the response pointer.

# 0.1.0
* Initial release
