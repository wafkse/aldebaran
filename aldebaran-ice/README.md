# Aldebaran ICE

ICEs, which stand for `Internal Compiler Error`, are crucial in the process of bug-finding and issue reporting, because they actually convey an error that has not been made obvious to the compiler. However, in other cases, this may be used to ignore faulting branches under the guise of "too improbable to realistically happen". Note that this is not necesarily bad, as many of these issues will be due to resource exhaustion.

Therefore, this crate aims to provide tiny, unintrusive, and quick-to-use utilities to, ideally, all compiler code.