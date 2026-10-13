## CI and release builds

GitHub Actions runs formatting checks, tests, and a release build on pushes, pull requests, and manual runs. Each run uploads a Linux x86_64 executable named `simple-http-linux-x86_64` as a workflow artifact. Push a version tag such as `v0.1.0` to also create a GitHub Release with that executable attached:

```sh
git tag v0.1.0
git push origin v0.1.0
```

The executable is built for Linux x86_64. Deploy it alongside a `pages/` directory or pass the site directory with `--pages-dir`.
