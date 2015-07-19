'''
parser8
    read line-by-line because I realized I'm an idiot
        alt is opening entire files
        old version stored entire dump in 1 string (lol)
        I managed to do it with a ~100GB pagefile, but it became unbearably slow (of course)
    uses SHA1 hash instead of timestamp for updating?
        hash more elegant
        if time is used, then script can calculate most recent article used
            when parsing new article, timestamp can be compared to this to determine if changed (valid?)
            otherwise, every hash would need to be compared with every other hash
                unless you could view edit log? that might be better
    maybe use a real argument parsing library
        --help  
        [1]     input file
        --output
        --update
        --chunks
        --chunk_size
    Reworking structure to lend itself to multithreading
        master thread extracts page, delegates finding links to children

    takes ~15 seconds for sample wiki
    Known Issues: Sometimes omits post metadata (ex: comment history) (???)
    Faster than the in-line version (speedup = ~2) and the newbie multithreading (speedup = ~8)
''' 

def get_contents(text, start, end):
    #lazy man's regex: retrieve text from a certain context
    #returns empty string if context absent or start == end
    #args: text to search in, beginning of context, end of context
    try:
        a = text.index(start)
        b = text.index(end, a)  #only look for end index after start index
    except ValueError:
        return ""
    else:
        return text[a+len(start):b]

import datetime, sys, os, re
start_time = datetime.datetime.now()

def parse_page(page_text):
    #match = re.search("<title>.+</title>", text)
    #title = match.group(0)[7:-8]
    title = get_contents(page_text, "<title>", "</title>")
    #write to error log if title == ""?
    hash = get_contents(page_text, "<sha1>", "</sha1>")
    output = "<page>\n" + title.upper() + "\n"
    output += hash + "\n"
    links = set(re.findall("\[\[[^]^\n]+\]\]", page_text))
    for link in links:
        link = link[2:-2].upper()
        if "|" in link:
            link = link[:link.index("|")]
        output += link + "\n"
    return output

#handle args
if len(sys.argv) == 2:
    #user-supplied input, generated output
    input_file = sys.argv[1]
    output_file = "out_%d.txt"
    output_num = 0
    while os.path.isfile(output_file % output_num):
        output_num += 1
    output_file = output_file % output_num
elif len(sys.argv) == 3:
    #user-supplied input and output
    input_file = sys.argv[1]
    output_file = sys.argv[2]
else:
    print "Usage: \""+sys.argv[0]+" input [output]\""
    exit()

output = open(output_file, "w")
links = []
parsed = 0
page = ""
with open(input_file) as input:
    for line in input:
        #page = ""
        page += line
        if "</page>" in line:
            output.write(parse_page(page))
            page = ""
            
output.close()

elapsed_time = datetime.datetime.now() - start_time
print "just read from: \t", input_file
print " and wrote to: \t ", output_file
print elapsed_time
print elapsed_time.seconds, "seconds"