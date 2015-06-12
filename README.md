# TextNonce

A nonce is a cryptographic concept of an arbitrary number that is never used more than once.

`TextNonce` is a nonce because the first 16 characters represents the current time, which
will never have been generated before, nor will it be generated again, across the period of
time in which Timespec is valid.

`TextNonce` additionally includes bytes of randomness, making it difficult to predict.
This makes it suitable to be used for a session ID.

It is also text-based, using only characters in the base64 character set.

Various length `TextNonce`es may be generated.  The minimum length is 16 characters, and
lengths must be evenly divisible by 4.
