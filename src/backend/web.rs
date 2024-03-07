use const_format::concatcp;
pub const HEADER_TEMPLATE: &str = concatcp!(
    "<!doctype html>\n",
    "<html>\n",
    "<head>\n",
    "    <link href=\"/assets/main.css\" rel=\"stylesheet\" />\n",
    "    <a href=\"/\">home</a> | <a href=\"/links\">links</a>\n",
    "    <title>url shortening</title>\n",
    "</head>\n",
    "<body>\n",
);
pub const FOOTER_TEMPLATE: &str = concatcp!(
    "\n<div class=\"footer\">&copy; 2024 chunski industries; omnia jura reservata, omnia pruna conservata</div>",
    "</body>\n",
    "</html>\n"
);
