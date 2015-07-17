'''parser6:
    designed to take a folder as input rather than a single file
    takes <15 minutes on sample wiki dump in 12 chunks
    takes a little over 5 hours on consolidated sample wiki
    uses very reasonable memory, so multithreading might just involve opening 4 files at a time
    functional-ish meh cli
    might port to C/C++ for performance increase (?) or continuity's sake (??)
        useful at least to test hash table with parsed sample data
'''   

import datetime, os, sys
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

start_time = datetime.datetime.now()        

if len(sys.argv) == 1:
    #args: input folder, outputt file
    input_folder = "E:\Libraries\Programs\C++_RPI\WikiLinkr\misc_data\out13"
    output = "output_7_5.txt"
else:
    input_folder = sys.argv[1]
    output = sys.argv[2]

files = os.listdir(input_folder)
fout = open(output, "w")    #TODO#

for filename in files:
    #cycle through each input file (must be in order unless broken up by pages)
    fin = open(os.path.join(input_folder, filename), "r").read()                    #read in file
    total_len = len(fin)
    offset_master = 0
    print "file: ", filename

    while offset_master < total_len:
        #cycle through each page in input file 
        (page, offset_master) = get_contents(fin, "<page>", "</page>", offset_master)
        if page == "":  #stop when no page is found (not after; should not add blank entry)
            break
        fout.write("<page>\n")
        title = get_contents(page, "<title>", "</title>")[0]
        fout.write(title.upper() + "\n")
        timestamp = get_contents(page, "<timestamp>", "</timestamp>")[0]
        fout.write("<timestamp>" + timestamp + "</timestamp>\n")

        link = " "
        offset = 0
        links = set()

        while offset <= len(page):
            #find all links; do not append blank link
            (link, offset) = get_contents(page, "[[", "]]", offset)
            if '|' in link:
                link = link[:link.index('|')]   #link is before pipe, name is after; link seems more consistent
            if link == "":
                break
            links.add(link.upper())            #append capitals: case may vary, but will probably affect hash function
        
        fout.write("\n".join(links) + "\n")

        fout.write("</page>\n")

fout.close()
    
elapsed_time = datetime.datetime.now() - start_time
print "just read from:\t\...", input_folder[len(input_folder)/3:]
print " and wrote to:\t...", output[len(output)/3:]
print elapsed_time
print elapsed_time.seconds, "seconds"
