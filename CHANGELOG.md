# 0.1.9
* **Breaking**: Adds `PhantomData<T>` in `LiminePtr<T>` to make the dropchk know that we own the `T`. This change does not effect
  the variance but changes the dropchk.
* Add `mmap_mut` to get a mutable reference to the memory map entries; useful when allocating physical memory prior to the 
  initialisation of the PMM ([#7](https://github.com/limine-bootloader/limine-rs/pull/7)).
* Fixed terminal column count returning row count and vice versa ([#8](https://github.com/limine-bootloader/limine-rs/pull/8)).

# 0.1.8
* Introduce the `into-uuid` feature which pulls in the `uuid` crate and implements conversion methods between `LimineUuid` and `uuid::Uuid` ([#3](https://github.com/limine-bootloader/limine-rs/pull/3)).

# 0.1.7
Yanked :boom:

# 0.1.6
* The `LimineFramebuffer`, `LimineTerminal` structures are updated to their new layout (`spec v3.5`).

# 0.1.5
* Fix the `LimineKernelFileRequest` request tag returning the wrong response ([#2](https://github.com/limine-bootloader/limine-rs/pull/2)).

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
