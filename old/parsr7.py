'''
parser7
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
    eventually add multi-threading
        master thread extracts page, delegates finding links to children
    takes ~30 seconds for sample wiki
    takes ~40 minutes for complete english wiki (24 with parsr8)
    
''' 

'''import argparse
parser = argparse.ArgumentParser(description="Extract link structure from Wikipedia dump")
parser.add_argument("input", type=string, )
args = parser.parse_args()'''

import datetime, sys, os, re
start_time = datetime.datetime.now()

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
with open(input_file) as input:
    for line in input:
        page = "<page>\n"
        #tags/metadata should be on their own line, so conditionals should be mutually exclusive
        if "<title>" in line:
            #look for title
            m = re.search("<title>.+</title>", line)
            title = m.group(0)[7:-8]
        elif "<sha1>" in line:
            #look for SHA1 hash (or maybe timestamp?)
            m = re.search("<sha1>[\w]+</sha1>", line)
            hash = m.group(0)[6:-7]
        elif "</page>" in line:
            #found end of page; write and reset
            page += title.upper() + "\n"
            page += hash + "\n"
            for match in set(links):
                #write link without brackets or repetitions
                #capitalize: caps may vary in context, but should be uniform for hash function later
                link = match[2:-2]
                if '|' in link:
                    #pipe means article_title|link_text_in_page; the latter varies by article
                    link = link[:link.index("|")]
                page += link.upper() + "\n"
            output.write(page)
            parsed += 1
            if parsed % 1000000 == 0:
                time = datetime.datetime.time(datetime.datetime.now())
                print "[" + str(time) +"] \t", parsed/1000000, "x 1M"
            #reset info for next page
            page = "<page>\n"
            title = hash = ""
            links = []
        else:
            #line is just regular text; look for links
            links += re.findall("\[\[[^]]+\]\]", line)
            
output.close()

elapsed_time = datetime.datetime.now() - start_time
print "just read from: \t", input_file
print " and wrote to: \t ", output_file
print elapsed_time
print elapsed_time.seconds, "seconds"