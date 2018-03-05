# uq

`uq` is a simple, user-friendly alternative to `sort | uniq`.

It removes duplicate lines from the output, regardless of the order.
Unlike `sort | uniq`, `uq` does not sort entries. This allows `uq` to operate on continuous streams as well.

```bash
$ python -c "while 1: print('a');print('b')" | uq
a
b
```