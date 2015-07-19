'''
parser9
    reworked version of parsr8 for multithreading
    the way it's currently set  up (spawn a thread per page) doesn't work
        way too much overhead (speedup of .25)
    lacking multithreading really isn't a big deal; runs once every time a dump is released, takes 40 minutes max
''' 

import datetime, sys, os, re
import multiprocessing as mp
start_time = datetime.datetime.now()

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

def parse_page(page_text, queue):
    #args: text to parse, mp.Queue(), file to write to, mp.Lock()
    #match = re.search("<title>.+</title>", text)
    #title = match.group(0)[7:-8]
    title = get_contents(page_text, "<title>", "</title>")
    #write to error log if title == ""?
    hash = get_contents(page_text, "<sha1>", "</sha1>")
    result = "<page>\n" + title.upper() + "\n"
    result += hash + "\n"
    links = set(re.findall("\[\[[^]^\n]+\]\]", page_text))
    for link in links:
        link = link[2:-2].upper()
        if "|" in link:
            link = link[:link.index("|")]
        result += link + "\n"

def listener(queue, out_name):
    out_file = open(out_name, "w")
    while 1:
        m = q.get()
        if m == "kill":
            break
        out_file.write(str(m) + "\n")
        out_file.flush()
    out_file.close()
    
def main():
    #handle args
    if len(sys.argv) == 2:
        #user-supplied input, generated output
        input_file = sys.argv[1]
        out_name = "out_%d.txt"
        out_num = 0
        while os.path.isfile(out_name % out_num):
            out_num += 1
        out_name = out_name % out_num
    elif len(sys.argv) == 3:
        #user-supplied input and output
        input_file = sys.argv[1]
        out_name = sys.argv[2]
    else:
        print "Usage: \""+sys.argv[0]+" input [output]\""
        exit()

    #out_file = open(out_name, "w")
    manager = mp.Manager()
    queue = manager.Queue()
    pool = mp.Pool(mp.cpu_count() + 2)
    watcher = pool.apply_async(listener, (queue, out_name))
    jobs = []
    page = ""
    with open(input_file) as input:
        for line in input:
            #page = ""
            page += line
            if "</page>" in line:
                job = pool.apply_async(parse_page, (page, queue))
                jobs.append(job)
                
                page = ""
    for job in jobs:
        job.get()
    
    queue.put('kill')
    pool.close()

    elapsed_time = datetime.datetime.now() - start_time
    print "just read from: \t", input_file
    print " and wrote to: \t ", out_name
    print elapsed_time
    print elapsed_time.seconds, "seconds"
    
    
if __name__ == '__main__':
    main()