#!/usr/bin/env python3
from __future__ import annotations
import argparse, hashlib, json, re
from pathlib import Path
POS={"n":"noun","v":"verb","adj":"adjective","adv":"adverb","pron":"pronoun","art":"article","prep":"preposition","conj":"conjunction"}
class AuthorityValidationError(ValueError): pass
def load_json(path: Path): return json.loads(path.read_text(encoding='utf-8'))
def map_part_of_speech(source_pos):
    if source_pos is None: return None
    try: return POS[source_pos.lower()]
    except KeyError as exc: raise AuthorityValidationError(f'unknown source part of speech: {source_pos}') from exc
def _clean_form(raw):
    text=' '.join(raw.split()).strip(); text=re.sub(r'\s*No other verb forms\.?\s*$','',text,flags=re.I).strip()
    if re.match(r'^\(\s*also\s+',text,flags=re.I):
        text=re.sub(r'^\(\s*also\s+','',text,flags=re.I).strip()
        if text.endswith(')'): text=text[:-1].strip()
    return text.strip(' ,.')
def _verb_form_sequence_from_word_cell(entry):
    if str(entry.get("part_of_speech") or "").lower() != "v":
        return None
    word_cell = str(entry.get("word_cell") or "")
    if not word_cell.strip():
        return None
    text = re.sub(r"\bNo\s+other\s+verb\s+forms\.?", "", word_cell, flags=re.I)
    sequence = []
    in_also_group = False
    for raw_line in text.splitlines():
        line = re.sub(r"\s*\(v\)\s*", "", raw_line, flags=re.I).strip()
        if not line:
            continue
        if re.match(r"^\(\s*also\s+", line, flags=re.I):
            in_also_group = True
            line = re.sub(r"^\(\s*also\s+", "", line, flags=re.I).strip()
        closes_also_group = in_also_group and line.endswith(")")
        if closes_also_group:
            line = line[:-1].rstrip()
        for segment in line.split(","):
            form = " ".join(segment.split()).strip(" ,.")
            if form.lower().startswith("also "):
                form = form[5:].strip()
            if form:
                sequence.append(form)
        if closes_also_group:
            in_also_group = False
    return sequence or None

def _verb_forms_from_word_cell(entry):
    sequence=_verb_form_sequence_from_word_cell(entry)
    if sequence is None: return None
    out=[]
    for form in sequence:
        if form not in out: out.append(form)
    return out

def normalize_forms(entry):
    source_verb_forms=_verb_forms_from_word_cell(entry)
    if source_verb_forms is not None: return source_verb_forms
    forms=entry.get("forms") or []
    if not forms: return [entry["headword"]]
    normalized=[]; seen=set()
    for raw in forms:
        form=_clean_form(str(raw))
        if not form or form in seen: continue
        seen.add(form); normalized.append(form)
    return normalized or [entry["headword"]]

def _verb_classification(entry):
    meaning=str(entry.get("meaning_or_alternatives") or "").lower()
    if "auxiliary modal verb" in meaning: return "defective_modal"
    if str(entry.get("headword") or "").upper()=="BE": return "irregular_auxiliary"
    return "lexical"

def derive_verb_paradigm(entry):
    sequence=_verb_form_sequence_from_word_cell(entry)
    if sequence is None: return None
    classification=_verb_classification(entry); base_form=sequence[0]
    if classification=="irregular_auxiliary":
        unique=[]
        for form in sequence[1:]:
            if form not in unique: unique.append(form)
        present=[f for f in unique if f.upper() in {"IS","ARE"}]
        past=[f for f in unique if f.upper() in {"WAS","WERE"}]
        participle=None
    elif classification=="defective_modal":
        present=sequence[1:2]; past=sequence[2:3]; participle=None
    else:
        present=sequence[1:2]; past=sequence[2:3]; participle=sequence[3] if len(sequence)>=4 else None
    return {"classification":classification,"source_sequence":sequence,"base_form":base_form,"simple_present_variants":present,"simple_past_variants":past,"past_participle":participle}

def _require_equal(label,actual,expected):
    if actual!=expected: raise AuthorityValidationError(f'{label} mismatch: got {actual!r}, expected {expected!r}')
def validate_authority(source,private_manifest,verified_manifest,dictionary,private_bundle_sha256):
    retained=verified_manifest['retained_source']; verified=verified_manifest['verified_ingest']; private_source=private_manifest['source']; counts=private_manifest['counts']
    _require_equal('issue',source.get('issue'),verified_manifest.get('issue')); _require_equal('publication date',source.get('publication_date'),verified_manifest.get('publication_date'))
    _require_equal('drive file id',source.get('drive_file_id'),retained.get('file_id')); _require_equal('source sha256',source.get('sha256'),retained.get('sha256')); _require_equal('source byte size',source.get('byte_size'),retained.get('byte_size')); _require_equal('physical page count',source.get('pdf_pages'),retained.get('physical_pages')); _require_equal('private bundle sha256',private_bundle_sha256,verified.get('private_bundle_sha256'))
    for key in ('issue','publication_date','drive_file_id','sha256','byte_size','pdf_pages'): _require_equal(f'private manifest source {key}',private_source.get(key),source.get(key))
    approved=sum(1 for e in dictionary if e.get('approved') is True); unapproved=sum(1 for e in dictionary if e.get('approved') is False)
    if approved+unapproved!=len(dictionary): raise AuthorityValidationError('dictionary contains a structural record without a boolean approved status')
    _require_equal('dictionary entry count',len(dictionary),verified.get('dictionary_entries')); _require_equal('private dictionary entry count',len(dictionary),counts.get('dictionary_entries')); _require_equal('approved structural record count',approved,verified.get('approved_headword_records')); _require_equal('approved structural private count',approved,counts.get('approved_headword_records')); _require_equal('unapproved structural record count',unapproved,verified.get('unapproved_headword_records')); _require_equal('unapproved structural private count',unapproved,counts.get('unapproved_headword_records')); _require_equal('source-declared approved word count',counts.get('source_declared_approved_words'),verified.get('source_declared_approved_words')); _require_equal('source-declared unapproved word count',counts.get('source_declared_unapproved_words'),verified.get('source_declared_unapproved_words'))
def validate_dictionary_artifact(dictionary_bytes,private_manifest,verified_manifest):
    private_artifact=private_manifest.get('artifacts',{}).get('dictionary.json',{}); verified_artifact_sha256=verified_manifest.get('verified_ingest',{}).get('artifact_sha256',{}).get('dictionary.json'); actual=hashlib.sha256(dictionary_bytes).hexdigest()
    _require_equal('private manifest dictionary.json bytes',len(dictionary_bytes),private_artifact.get('bytes')); _require_equal('private manifest dictionary.json sha256',actual,private_artifact.get('sha256')); _require_equal('verified manifest dictionary.json sha256',actual,verified_artifact_sha256)
def compile_entry(index,entry):
    compiled={'lemma':entry['headword'],'status':'approved' if entry['approved'] else 'unapproved','part_of_speech':map_part_of_speech(entry.get('part_of_speech')),'forms':normalize_forms(entry),'senses':entry.get('senses',[]),'alternatives':entry.get('alternatives',[]),'restrictions':entry.get('restrictions',[]),'interpretation_state':entry.get('interpretation_state','structural'),'provenance':{'structural_record_index':index,'source_pages':entry.get('source_pages',[])},'source_semantics':{'word_cell':entry.get('word_cell',''),'meaning_or_alternatives':entry.get('meaning_or_alternatives',''),'ste_example':entry.get('ste_example',''),'non_ste_example':entry.get('non_ste_example','')}}
    if entry.get('approved') is True:
        p=derive_verb_paradigm(entry)
        if p is not None: compiled['verb_paradigm']=p
    return compiled
def compile_document(authority_dir,verified_manifest_path,private_bundle_sha256):
    source=load_json(authority_dir/'source.json'); pm=load_json(authority_dir/'manifest.json'); dp=authority_dir/'dictionary.json'; db=dp.read_bytes(); d=json.loads(db.decode('utf-8')); vm=load_json(verified_manifest_path)
    validate_dictionary_artifact(db,pm,vm); validate_authority(source,pm,vm,d,private_bundle_sha256); v=vm['verified_ingest']
    return {'metadata':{'standard':vm['standard'],'issue':vm['issue'],'date':vm['publication_date'],'scope':'issue9_private_authority_runtime','authority':{'drive_file_id':source['drive_file_id'],'source_sha256':source['sha256'],'source_byte_size':source['byte_size'],'physical_pages':source['pdf_pages'],'private_bundle_sha256':private_bundle_sha256},'dictionary_cardinalities':{'source_declared_approved_words':v['source_declared_approved_words'],'source_declared_unapproved_words':v['source_declared_unapproved_words'],'structural_approved_records':v['approved_headword_records'],'structural_unapproved_records':v['unapproved_headword_records']}},'entries':[compile_entry(i,e) for i,e in enumerate(d)]}
def render_document(document): return json.dumps(document,ensure_ascii=False,indent=2)+'\n'
def write_document(document,path): path.parent.mkdir(parents=True,exist_ok=True); tmp=path.with_name(path.name+'.tmp'); tmp.write_text(render_document(document),encoding='utf-8'); tmp.replace(path)
if __name__=='__main__':
    p=argparse.ArgumentParser(); p.add_argument('--authority-dir',type=Path,required=True); p.add_argument('--verified-manifest',type=Path,required=True); p.add_argument('--private-bundle-sha256',required=True); p.add_argument('--out',type=Path,required=True); a=p.parse_args(); write_document(compile_document(a.authority_dir,a.verified_manifest,a.private_bundle_sha256),a.out)
