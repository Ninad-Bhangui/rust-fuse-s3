# rust-fuse-s3 (W.I.P)
Attempt at mounting s3 buckets using FUSE in rust

Currently stuck with:

Fuse trait I'm implementing does not expect an async function but I need an async function to work with s3.
