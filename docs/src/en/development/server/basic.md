# Basic Development Precautions

## Scripts

We provide a series of scripts to assist you with routine tasks.

You can run `script/test_server.py` to execute server-side tests. To customize your settings, you can create a `local` directory (which is already added to `.gitignore`) and then copy the script into it.

For example, we have set the `OURCHAT_CONFIG_FILE` environment variable to read the configuration file for server execution. You can copy `script/test_server.py` to the `local` directory, copy configuration files like `ourchat.toml`, and then add:

```python
os.putenv("OURCHAT_CONFIG_FILE", "../local/ourchat.toml")
```

to customize the server-side testing.

## Testing

Due to the particularity of server-side testing, we have introduced a `test_lib` module to assist with testing, which you can refer to in existing unit tests.

## Documentation

Please make good use of `cargo doc`, as we have provided you with comprehensive documentation! Call `cargo doc --document-private-items` to generate private documentation.
