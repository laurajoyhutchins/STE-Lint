#!/usr/bin/env python3
from __future__ import annotations
import argparse, hashlib, json, re, sqlite3, subprocess
from pathlib import Path
import pdfplumber
from pypdf import PdfReader

POS_RE = re.compile(r"\((adj|adv|art|conj|n|prep|pron|v)\)", re.I)
RULE_RE = re.compile(r"^\s*Rule\s+(\d+\.\d+)\s+(.*)$", re.I)
GR_RE = re.compile(r"^\s*GR-(\d+)\s+(.*)$", re.I)
SECTION_SUMMARY = {1:45,2:63,3:67,4:77,5:87,6:95,7:103,8:107,9:115}
SPECIAL_HEADWORDS = {"FOR EXAMPLE", "such as"}
ODD_X = [72.0,176.4,306.0,435.7,565.3]
EVEN_X = [50.4,154.8,284.4,414.1,543.7]
EXPECTED_SHA256 = 'd1f4ea9e7cd6e46b47aa9057209f99e78c0e9cfc4e27a5b07895b05c1a166431'
EXPECTED_BYTES = 3316157
EXPECTED_PAGES = 434
TABLE_SETTINGS_BASE = dict(
    vertical_strategy="explicit", horizontal_strategy="lines",
    snap_tolerance=2, join_tolerance=3, intersection_tolerance=3, text_tolerance=2,
)


def sha256_file(path: Path) -> str:
    h=hashlib.sha256()
    with path.open('rb') as f:
        for chunk in iter(lambda:f.read(1<<20), b''): h.update(chunk)
    return h.hexdigest()

def clean_lines(lines):
    out=[]
    for line in lines:
        s=line.rstrip()
        st=s.strip()
        if not st: out.append(""); continue
        if st == "ASD-STE100 Simplified Technical English": continue
        if re.match(r"^(Issue 9|2025-01-15)$", st): continue
        if re.search(r"Part 1 - Writing [Rr]ules", st) and "Page" in st: continue
        if re.search(r"Part 2 - Dictionary", st) and "Page" in st: continue
        if re.match(r"^Page\s+[12]-", st): continue
        out.append(s)
    while out and not out[0].strip(): out.pop(0)
    while out and not out[-1].strip(): out.pop()
    return out

def logical_page_label(text: str):
    m=re.search(r"\bPage\s+([12]-[0-9A-Za-z-]+)", text)
    return m.group(1) if m else None

def extract_pages(pdf: Path, outdir: Path):
    txt=outdir/'issue9-layout.txt'
    subprocess.run(['pdftotext','-layout',str(pdf),str(txt)],check=True)
    parts=txt.read_text(errors='replace').split('\f')
    if parts and not parts[-1].strip(): parts=parts[:-1]
    return parts

def extract_rules(pages):
    occ={}
    for pn in range(43,129):
        lines=pages[pn-1].splitlines()
        for idx,line in enumerate(lines):
            m=RULE_RE.match(line)
            if m: occ.setdefault(m.group(1),[]).append((pn,idx,m.group(2).strip()))
    ordered=[]
    for major in range(1,10):
        ids=sorted([rid for rid in occ if rid.startswith(f'{major}.')], key=lambda x:int(x.split('.')[1]))
        summary=SECTION_SUMMARY[major]
        for rid in ids:
            cands=[o for o in occ[rid] if o[0]>=summary]
            start=cands[1] if len(cands)>1 else cands[0]
            ordered.append((rid,start))
    seen=set(); starts=[]
    for rid,start in ordered:
        if rid not in seen:
            starts.append((rid,start)); seen.add(rid)
    rules=[]
    for i,(rid,(sp,si,title0)) in enumerate(starts):
        ep,ei = (starts[i+1][1][0], starts[i+1][1][1]) if i+1<len(starts) else (123,0)
        chunks=[]
        for pn in range(sp,ep+1):
            lines=pages[pn-1].splitlines()
            a=si if pn==sp else 0
            b=ei if pn==ep else len(lines)
            chunks.extend(clean_lines(lines[a:b]))
        title_parts=[title0]
        lines=pages[sp-1].splitlines(); j=si+1
        while j<len(lines) and lines[j].strip() and not lines[j].lstrip().startswith(('Examples:', 'Example:', 'A ', 'The ', 'In ', 'You ', 'Use ', 'Do ', 'This ', 'If ', 'When ', 'STE ')):
            t=lines[j].strip()
            if len(t)>110: break
            title_parts.append(t); j+=1
        title=' '.join(x for x in title_parts if x).strip()
        body='\n'.join(chunks).strip()
        rules.append({'id':rid,'section':int(rid.split('.')[0]),'title':title,'start_pdf_page':sp,'end_pdf_page':ep if ei>0 else ep-1,'logical_page_start':logical_page_label(pages[sp-1]),'text':body})
    return rules

def extract_grs(pages):
    starts=[]
    for pn in range(123,128):
        lines=pages[pn-1].splitlines()
        for idx,line in enumerate(lines):
            m=GR_RE.match(line)
            if m: starts.append((int(m.group(1)),pn,idx,m.group(2).strip()))
    grs=[]
    for i,(num,sp,si,title) in enumerate(starts):
        ep,ei=(starts[i+1][1],starts[i+1][2]) if i+1<len(starts) else (128,0)
        chunks=[]
        for pn in range(sp,ep+1):
            lines=pages[pn-1].splitlines(); a=si if pn==sp else 0; b=ei if pn==ep else len(lines)
            chunks.extend(clean_lines(lines[a:b]))
        grs.append({'id':f'GR-{num}','title':title,'start_pdf_page':sp,'end_pdf_page':ep if ei>0 else ep-1,'text':'\n'.join(chunks).strip()})
    return grs

def table_settings(pn):
    d=TABLE_SETTINGS_BASE.copy(); d['explicit_vertical_lines']=ODD_X if pn%2 else EVEN_X; return d

def is_new_headword(cell):
    c=(cell or '').strip()
    return bool(c and (POS_RE.search(c) or c in SPECIAL_HEADWORDS))

def normalize_headword(raw):
    raw=' '.join(raw.replace('\n',' ').split())
    if raw in SPECIAL_HEADWORDS: return raw, None, []
    m=POS_RE.search(raw); pos=m.group(1).lower() if m else None
    if m:
        head=raw[:m.start()].strip().rstrip(',')
    else:
        head=raw
    head=head.split(',')[0].strip()
    forms=[]
    if pos=='v':
        pieces=[p.strip(' ,.') for p in re.split(r'[,\n]+', raw) if p.strip()]
        for p in pieces:
            p=POS_RE.sub('',p).strip()
            if p and p.lower() not in {'no other verb forms'} and not p.startswith('No other'):
                forms.append(p)
    return head,pos,forms

def approved_from_headword(head):
    letters=''.join(ch for ch in head if ch.isalpha())
    return bool(letters and letters.upper()==letters)

def extract_dictionary(pdf: Path):
    raw_rows=[]; entries=[]; current=None
    with pdfplumber.open(pdf) as doc:
        for pn in range(149,435):
            table=doc.pages[pn-1].extract_table(table_settings(pn))
            if not table: continue
            for ri,row in enumerate(table):
                row=[(x or '').strip() if x is not None else None for x in row]
                raw_rows.append({'pdf_page':pn,'row_index':ri,'cells':row})
                c0=(row[0] or '').strip()
                if is_new_headword(c0):
                    if current: entries.append(current)
                    current={'headword_raw':c0,'fragments':[],'source_pages':[]}
                elif c0 and current:
                    current['headword_raw'] += '\n' + c0
                if current:
                    current['fragments'].append({'pdf_page':pn,'row_index':ri,'cells':row})
                    if pn not in current['source_pages']: current['source_pages'].append(pn)
    if current: entries.append(current)
    for e in entries:
        head,pos,forms=normalize_headword(e['headword_raw'])
        e['headword']=head; e['part_of_speech']=pos; e['forms']=forms; e['approved']=approved_from_headword(head)
        cols=[[],[],[],[]]
        for f in e['fragments']:
            for i,c in enumerate(f['cells']):
                if c: cols[i].append(c)
        e['word_cell']='\n'.join(cols[0]); e['meaning_or_alternatives']='\n'.join(cols[1]); e['ste_example']='\n'.join(cols[2]); e['non_ste_example']='\n'.join(cols[3])
    return raw_rows,entries

def write_json(path,obj):
    path.write_text(json.dumps(obj,ensure_ascii=False,indent=2)+"\n")

def build_sqlite(path, source, pages, rules, grs, entries, raw_rows):
    if path.exists(): path.unlink()
    con=sqlite3.connect(path)
    con.executescript('''
    create table source(key text primary key, value text not null);
    create table pages(pdf_page integer primary key, logical_page text, text text not null, text_sha256 text not null);
    create table rules(id text primary key, section integer, title text, start_pdf_page integer, end_pdf_page integer, text text);
    create table recommendations(id text primary key, title text, start_pdf_page integer, end_pdf_page integer, text text);
    create table dictionary_entries(id integer primary key, headword text, headword_raw text, part_of_speech text, approved integer, forms_json text, meaning_or_alternatives text, ste_example text, non_ste_example text, source_pages_json text);
    create table dictionary_rows(pdf_page integer,row_index integer,cells_json text,primary key(pdf_page,row_index));
    ''')
    con.executemany('insert into source values (?,?)',[(k,json.dumps(v,ensure_ascii=False)) for k,v in source.items()])
    con.executemany('insert into pages values (?,?,?,?)',[(i+1,logical_page_label(t),t,hashlib.sha256(t.encode()).hexdigest()) for i,t in enumerate(pages)])
    con.executemany('insert into rules values (?,?,?,?,?,?)',[(r['id'],r['section'],r['title'],r['start_pdf_page'],r['end_pdf_page'],r['text']) for r in rules])
    con.executemany('insert into recommendations values (?,?,?,?,?)',[(r['id'],r['title'],r['start_pdf_page'],r['end_pdf_page'],r['text']) for r in grs])
    con.executemany('insert into dictionary_entries values (?,?,?,?,?,?,?,?,?,?)',[(i+1,e['headword'],e['headword_raw'],e['part_of_speech'],int(e['approved']),json.dumps(e['forms'],ensure_ascii=False),e['meaning_or_alternatives'],e['ste_example'],e['non_ste_example'],json.dumps(e['source_pages'])) for i,e in enumerate(entries)])
    con.executemany('insert into dictionary_rows values (?,?,?)',[(r['pdf_page'],r['row_index'],json.dumps(r['cells'],ensure_ascii=False)) for r in raw_rows])
    con.commit(); con.close()

def main():
    ap=argparse.ArgumentParser(); ap.add_argument('pdf',type=Path); ap.add_argument('--out',type=Path,required=True); args=ap.parse_args()
    out=args.out; out.mkdir(parents=True,exist_ok=True)
    pages=extract_pages(args.pdf,out)
    reader=PdfReader(str(args.pdf))
    meta={str(k).lstrip('/'):str(v) for k,v in (reader.metadata or {}).items()}
    source={'title':'ASD-STE100 Simplified Technical English','issue':9,'publication_date':'2025-01-15','drive_file_id':'1GfSldRfzXs91pG1BbgLjbzJFJML_wifP','mime_type':'application/pdf','byte_size':args.pdf.stat().st_size,'sha256':sha256_file(args.pdf),'pdf_pages':len(reader.pages),'encrypted':bool(reader.is_encrypted),'metadata':meta}
    if source['sha256'] != EXPECTED_SHA256 or source['byte_size'] != EXPECTED_BYTES or source['pdf_pages'] != EXPECTED_PAGES:
        raise SystemExit('source identity mismatch: refusing Issue 9 ingest')
    rules=extract_rules(pages); grs=extract_grs(pages); raw_rows,entries=extract_dictionary(args.pdf)
    page_records=[{'pdf_page':i+1,'logical_page':logical_page_label(t),'text':t,'text_sha256':hashlib.sha256(t.encode()).hexdigest()} for i,t in enumerate(pages)]
    write_json(out/'source.json',source); write_json(out/'rules.json',rules); write_json(out/'general-recommendations.json',grs); write_json(out/'dictionary.json',entries)
    with (out/'pages.jsonl').open('w') as f:
        for r in page_records: f.write(json.dumps(r,ensure_ascii=False)+'\n')
    with (out/'dictionary-rows.jsonl').open('w') as f:
        for r in raw_rows: f.write(json.dumps(r,ensure_ascii=False)+'\n')
    build_sqlite(out/'issue9-authority.sqlite3',source,pages,rules,grs,entries,raw_rows)
    approved=[e for e in entries if e['approved']]
    validations={
        'source_identity_matches':source['sha256']==EXPECTED_SHA256 and source['byte_size']==EXPECTED_BYTES and source['pdf_pages']==EXPECTED_PAGES,
        'pdf_page_count_434':len(reader.pages)==434,
        'rules_count_53':len(rules)==53,
        'general_recommendations_count_8':len(grs)==8,
        'dictionary_entries':len(entries),
        'approved_entry_count':len(approved),
        'standard_states_875_approved_words':875,
        'standard_states_1274_unapproved_words':1274,
        'raw_dictionary_rows':len(raw_rows),
        'dictionary_pages_without_table':[pn for pn in range(149,435) if not any(r['pdf_page']==pn for r in raw_rows)],
    }
    manifest={'source':source,'counts':{'pages':len(pages),'rules':len(rules),'general_recommendations':len(grs),'dictionary_entries':len(entries),'approved_entries':len(approved),'unapproved_entries':len(entries)-len(approved),'dictionary_rows':len(raw_rows)},'validations':validations,'artifacts':{}}
    for name in ['source.json','rules.json','general-recommendations.json','dictionary.json','pages.jsonl','dictionary-rows.jsonl','issue9-authority.sqlite3','issue9-layout.txt']:
        p=out/name; manifest['artifacts'][name]={'bytes':p.stat().st_size,'sha256':sha256_file(p)}
    write_json(out/'manifest.json',manifest)
    print(json.dumps(manifest,indent=2))
if __name__=='__main__': main()
