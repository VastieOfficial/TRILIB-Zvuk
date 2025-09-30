# TRILib-ZVUK
A part of Vastie TRack Information LIBrary, that downloads music directly from Zvuk (without third-party providers)

# Legal notice
This tool is provided for personal, non-commercial use only. In Russia, personal copying of lawfully obtained sound recordings is permitted under [Article 1273 of the Civil Code](https://www.consultant.ru/document/cons_doc_LAW_64629/f63562ebf49f4d5fbe0c3daa9ea22a689d2d64ab/). In the United States, the [Audio Home Recording Act](https://www.congress.gov/bill/102nd-congress/senate-bill/1623) allows private copying of legally obtained music, but other laws (including the DMCA) prohibit bypassing DRM or downloading copyrighted content without authorization. You are solely responsible for ensuring your use complies with local laws.

# Usage

1. Set enviveroment variables:

| Variable         | Value                                                     
| ---------------: | --------------------------------------------------------- 
| TRI_CACHE        | Path to Trilib's cache (any folder, default CWD/TRICACHE) 
| TRI_ZVUK_PORT | HTTP port (default 3501)                                  
1. Run / build: `cargo run`
2. POST Request JSON payload (escape Unicode) to `/dl`:
Either URL or Title must be specified.

| Key              | Value                                                                                                     
| ---------------: | --------------------------------------------------------------------------------------------------------- 
| id              | ID of the ZVUK track
| hash             | Hash of the track (coming from TRIlib, any string that doesn't violate filesystem's restrictions)                                                                                                                  
| auth_cookie            | Your login cookies
3. Done! Your track will be saved to TRI_CACHE/hash/zvuk/[best/mid].[extenstion]

# License
This software is released under MIT license. 
