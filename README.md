# zcashrpc

This cargo package provides a few libraries useful for implementing a
`zcashd` RPC client using the `tokio` async I/O framework. It also
includes some utility binaries.

## License

This code is licensed under the MIT license. See `./LICENSE.txt`.

## Security & Safety

This code has not been audited, and is currently a part time hobby
project.

## API Design

The client provides a set of async methods corresponding to zcash RPC
methods.

This project does not aim to support all zcash RPC methods! Only a
subset requested by users. The rationale is that not all of the zcash
RPC methods are actually used by downstream code and we want to avoid
adding unnecessary technical debt.
