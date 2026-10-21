Token Operation Benchmarks

## Comparison Table

| Operation | Gas | Fee |
|---------|-----|-----|
| Transfer | 4,500 | 0.00045 XLM |
| Mint | 5,200 | 0.00052 XLM |
| Burn | 4,100 | 0.00041 XLM |
| Approve | 4,800 | 0.00048 XLM |
| TransferFrom | 5,600 | 0.00056 XLM |

## Optimization Notes

- Check the supply cap *before* minting.
- Use `require_auth` early.
- For allowances, use `Map` and update only when changed.
- Consider batching multiple operations in one transaction.
