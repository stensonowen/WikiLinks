'''parser5:
    designed to take a folder as input rather than a single file
    takes <15 minutes on sample wiki dump
    uses very reasonable memory, so multithreading might just involve opening 4 files at a time
'''   

import datetime, os
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
input_folder = "E:\Libraries\Programs\C++_RPI\WikiLinkr\misc_data\out5"
output = "output_5.txt"
#fout = open(output, "w")
#input = r"E:\Libraries\Downloads\WIKIPEDIA\misc_data\simplewiki-20150603-pages-articles.xml"
#input = r"E:\Libraries\Downloads\WIKIPEDIA\misc_data\simple_sample.txt"
#input = r"E:\Libraries\Programs\C++_RPI\WikiLinkr\misc_data\disk1.gsd"

files = os.listdir(input_folder)


for filename in files:
    #cycle through each input file (must be in order unless broken up by pages)
    fin = open(os.path.join(input_folder, filename), "r").read()                    #read in file
    fout = open(os.path.join(input_folder, filename[:-4]+"_out.txt"), "w")     #prep out file
    total_len = len(fin)
    offset_master = 0
    print "file: ", filename
    carry_over = ""    

    while offset_master < total_len:
        #cycle through each page in input file 
        (page, offset_master) = get_contents(fin, "<page>", "</page>", offset_master)
        if page == "":  #stop when no page is found (not after; should not add blank entry)
            break
        page = carry_over + page
        fout.write("<page>\n")
        title = get_contents(page, "<title>", "</title>")[0]
        fout.write(title.upper() + "\n")
        timestamp = get_contents(page, "<timestamp>", "</timestamp>")[0]
        fout.write("<timestamp>" + timestamp + "</timestamp>\n")

        link = " "
        offset = 0
        links = set()

        page_len = len(page)    #does this have any performance benefit?
        while offset <= page_len:
            #find all links; do not append blank link
            (link, offset) = get_contents(page, "[[", "]]", offset)
            if '|' in link:
                link = link[:link.index('|')]   #link is before pipe, name is after; link seems more consistent
            if link == "":
                break
            links.add(link.upper())            #append capitals: case may vary, but will probably affect hash function

        carry_over = page[offset:]
        #for link in links:
            #fout.write(link + "\n")
        fout.write("\n".join(links) + "\n")

        fout.write("</page>\n")

    
elapsed_time = datetime.datetime.now() - start_time
print elapsed_time
print elapsed_time.seconds, "seconds"
