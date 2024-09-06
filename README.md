# Link scraper

This small university-project aims to scrape links from the contents of any file.
It also provides some extra information about the location of those links in the file,
and what role they have.

Currently, it supports most commonly used document formats, all text-based formats,
and can provide extra information for xml-based ones. A complete list of all supported formats can
be found [here](#supported-formats).<br/>
It also contains a [convenience function](src/any_format_scraper.rs), that just takes any file and tries to guess the correct filetype for you.

This crate is heavily seperated into features,
to avoid blowing up its size if you only need it for a small amount of known file-types.<br/>
By default, only the `any_format` feature is active, that can _not_ be used on its own.
So to actually use this crate you __need__ to activate at least one format-feature.

### Supported formats

 - TXT 
 - PDF
 - DOCX
 - PPTX
 - XLSX
 - ODP
 - ODS
 - ODT
 - OTT
 - RTF
 - XML ( And all xml-based formats. Also has some extra features for the following xml-based formats )
   - SVG
   - XLink (There is the beginnings of an XLink-parser/validator in here. It is not this crates' purpose, but since I couldn't really find any other crate that does this, I thought I'd mention it)
 - Image formats (From exif-data)
   - JPG / JPEG
   - PNG
   - WebP
   - TIFF
   - HEIF
   
### Any format scraper

This modules' `scrape`-function will behave nicely with most files, however its ability to recognize filetypes is 
somewhat limited, and if you know what format you're using, you should probably use the format-specific module's `scrape`-function instead.

## Known issues

### Error when trying to use the crate under Windows with PDF enabled

The build process throws an error
```
error: failed to run custom build command for `mupdf-sys v0.4.4`
note: To improve backtraces for build dependencies, set the CARGO_PROFILE_DEV_BUILD_OVERRIDE_DEBUG=true environment variable to enable debug information generation.
Caused by:
  process didn't exit successfully: `C:\path\to\your\project\target\debug\build\mupdf-sys-0000000000000000\build-script-build` (exit code: 101)
  --- stderr
  thread 'main' panicked at 
...
```

This is due to an [issue with the mupdf-crate](https://github.com/messense/mupdf-rs/issues/72) related to the filesize-restrictions of crates.io.

As a temporary workaround you can add the mupdf-git-repository to your project as a submodule with
```bash
git submodule add https://github.com/messense/mupdf-rs.git external/mupd
```

Don't forget to initialize the submodule with
```bash
git submodule update --init --recursive
```

And add the following lines to your `Cargo.toml`:
```toml
[patch.crates-io]
mupdf = { path= "external/mupdf" }
```

## License

This work is released under the GPLv3 license. A copy of the license is provided in the [LICENSE](./LICENSE) file.