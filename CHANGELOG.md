# 0.1.3
* Fix the broken layout of the `LimineTerminal` structure.

# 0.1.2
* **Breaking**: The `write` function now takes a `&LimineTerminal` as an argument as expected. In addition to that, the
                `write` function returns an `Option` containg the writer helper closure function since a faulty bootloader *can*
                return null terminal write function pointer.

# 0.1.1
* **Breaking**: The `response` field for the request structures is now private and the `get_response` function must be used instead to retrieve the response pointer.

# 0.1.0
* Initial release
