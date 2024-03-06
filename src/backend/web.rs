use const_format::concatcp;
pub const HEADER_TEMPLATE: &str = concatcp!(
    "<!doctype html>",
    "<html>",
    "<head>",
    "    <link href=\"/assets/main.css\" rel=\"stylesheet\" />",
    "    <a href=\"/\">home</a> | <a href=\"/links\">links</a>",
    "    <title>url shortening</title>",
    "</head>",
);
pub const FOOTER_TEMPLATE: &str = concatcp!(
    "<div class=\"footer\">&copy; 2024 chunski industries; omnia jura reservata, omnia pruna conservata</div>",
    "</html>"
);
