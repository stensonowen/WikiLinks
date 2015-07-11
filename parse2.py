def get_contents(text, start, end, offset=0):
    #poor man's regex: retrieve text from a certain context
    #returns empty string if context absent or start == end
    #args: text to search in, beginning of context, 
        #end of context, where in text to start searching
    text = text[offset:]
    try:
        a = text.index(start)
        b = text.index(end, a)  #only look for end index after start index
    except ValueError:
        return ("", offset)
    else:
        return (text[a+len(start):b], b+len(end)+offset)
    

input = r"E:\Libraries\Downloads\WIKIPEDIA\misc_data\simplewiki-20150603-pages-articles.xml"
output = "output.txt"
fin = open(input, "r").read()
fout = open(output, "w")

offset_master = 0
total_len = len(fin)

while offset_master < total_len:
#for i in range(10):


    #page = fin[fin.index("<page>"):fin.index("</page>")]
    (page, offset_master) = get_contents(fin, "<page>", "</page>", offset_master)
    fout.write("<page>\n")
    title = get_contents(page, "<title>", "</title>")[0]
    fout.write(title.upper() + "\n")
    timestamp = get_contents(page, "<timestamp>", "</timestamp>")[0]
    fout.write("<timestamp>" + timestamp + "</timestamp>\n")


    link = " "
    offset = 0
    links = set()

    page_len = len(page)
    while offset <= page_len:
        #find all links; do not push back blank link
        (link, offset) = get_contents(page, "[[", "]]", offset)
        if '|' in link:
            link = link[:link.index('|')]
        if link == "":
            break
        links.add(link.upper())
        #append capitals: case may vary, but will probably affect hash function

    for link in links:
        #if link.isspace() == False:
        #fout.write(link.strip("\n") + "\n")
        fout.write(link + "\n")

    fout.write("</page>\n")