#ifdef EECH_CORE_TERRAIN_CRT_OBSERVER
/* Opt-in observation of the REAL module CRT. Integer bookkeeping only in hooks;
 * no output, allocation, time, extra RNG call, physical answer or global reset.
 * Capture through first completed load and next seed; later calls still forward.
 * One owner thread/process, like the engine. Overflow is an explicit report error. */
#include "eech_core_crt_observer.h"
#undef rand
#undef srand
#include <stdint.h>
#include <stdio.h>
#include <string.h>
#include <stdarg.h>
#include <errno.h>

#define SITES 64
#define MARKS 64
#define SKINS 128
typedef struct { const char *file; int line; uint64_t count[3], first[3], last; int first_value, last_value, gap_scene; } site_row;
typedef struct { const char *kind; uint64_t order, draws; int a,b,c; } mark_row;
typedef struct { int scene,animation,season,normal,desert,winter,ctor,frames; uint64_t calls,draws; } skin_row;
static site_row sites[SITES]; static mark_row marks[MARKS]; static skin_row skins[SKINS];
static int site_count,mark_count,skin_count,phase,loads,ends,seeds,problem,ctor_kind,skin_at=-1;
static uint64_t order,draws,phase_draws[3];
static uint32_t digest=2166136261u;

static void mark (const char *kind,int a,int b,int c)
{
    if (mark_count==MARKS) { problem=1;return; }
    marks[mark_count++]=(mark_row){kind,++order,draws,a,b,c};
}
static int site (const char *file,int line)
{
    int i;
    for(i=0;i<site_count;i++) if(sites[i].line==line && !strcmp(sites[i].file,file)) return i;
    if(site_count==SITES) { problem=1;return -1; }
    i=site_count++;sites[i].file=file;sites[i].line=line;return i;
}
int eech_core_crt_rand (const char *file,int line)
{
    int value=rand(); /* the ONLY underlying call; even after freeze/error */
    int saved_errno=errno;
    if(phase<3) {
        int i=site(file,line); ++order;++draws;++phase_draws[phase];
        digest=(digest^(uint32_t)value)*16777619u;digest=(digest^(uint32_t)(i+1))*16777619u;
        if(i>=0) { site_row *s=&sites[i]; if(!s->first[phase]) {s->first[phase]=order;if(phase==2) s->gap_scene=skin_at>=0?skins[skin_at].scene:-1;} if(!s->last) s->first_value=value;s->last=order;s->last_value=value;++s->count[phase]; }
        if(skin_at>=0) ++skins[skin_at].draws;
    }
    errno=saved_errno;return value;
}
void eech_core_crt_srand (unsigned int seed,const char *file,int line)
{
    int saved_errno;
    srand(seed); /* same conversion, seed and exactly ONE original call */
    saved_errno=errno;
    if(phase<3) { (void)site(file,line);++seeds;mark("seed",(int)seed,line,phase);if(phase==2) phase=3; }
    errno=saved_errno;
}
void eech_core_crt_load_begin (int clouds,int season)
{
    if(phase>=3) return;
    if(phase!=0 || loads || !seeds) problem=1;
    ++loads;mark("load_begin",clouds,season,0);phase=1;
}
void eech_core_crt_load_end (int map,int width,int height)
{
    if(phase>=3) return;
    if(phase!=1 || ctor_kind || skin_at>=0) problem=1;
    ++ends;mark("load_end",map,width,height);phase=2;
}
void eech_core_crt_boundary (const char *name)
{
    if(phase<3) mark(name,0,0,0);
}
void eech_core_crt_ctor (int kind,int scene)
{
    if(phase>=3) return;
    if(kind==0) { if(!ctor_kind || skin_at>=0) problem=1;mark("ctor_end",ctor_kind,scene,0);ctor_kind=0; }
    else { if(ctor_kind || (kind!=1 && kind!=2)) problem=1;ctor_kind=kind;mark("ctor_begin",kind,0,0); }
}
void eech_core_crt_skin_begin (int scene,int animation,int season,int normal,int desert,int winter)
{
    int i;
    if(phase>=3) return;
    if(skin_at>=0) problem=1;
    for(i=0;i<skin_count;i++) if(skins[i].scene==scene && skins[i].animation==animation && skins[i].season==season && skins[i].ctor==ctor_kind) break;
    if(i==SKINS) {problem=1;return;}
    if(i==skin_count) {++skin_count;skins[i]=(skin_row){scene,animation,season,normal,desert,winter,ctor_kind,-1,0,0};}
    skin_at=i;++skins[i].calls;
}
void eech_core_crt_skin_frames (int frames)
{
    if(phase>=3) return;
    if(skin_at<0) problem=1;else skins[skin_at].frames=frames;
}
void eech_core_crt_skin_end (void)
{
    if(phase>=3) return;
    if(skin_at<0) problem=1;skin_at=-1;
}
static int append (char **out,int *left,const char *format,...)
{
    va_list args;int n;
    va_start(args,format);n=vsnprintf(*out,(size_t)*left,format,args);va_end(args);
    if(n<0 || n>=*left) return -1;
    *out+=n;*left-=n;return 0;
}
static const char *basename_of (const char *file)
{
    const char *base=file;for(;*file;file++) if(*file=='/' || *file=='\\') base=file+1;return base;
}
int eech_core_crt_report (char *text,int capacity)
{
    char *out=text;int left=capacity,i;
    if(!text || capacity<=0) return -1;
    text[0]=0;
#define ADD(...) do {if(append(&out,&left,__VA_ARGS__)) return -1;} while(0)
    ADD("core-crt-summary schema=1 complete=%d problem=%d loads=%d ends=%d seeds=%d draws=%llu pre=%llu load=%llu gap=%llu digest=0x%08x sites=%d marks=%d skins=%d\n",phase==3,problem,loads,ends,seeds,(unsigned long long)draws,(unsigned long long)phase_draws[0],(unsigned long long)phase_draws[1],(unsigned long long)phase_draws[2],digest,site_count,mark_count,skin_count);
    for(i=0;i<mark_count;i++) {mark_row *m=&marks[i];ADD("core-crt-mark kind=%s order=%llu draws=%llu a=%d b=%d c=%d\n",m->kind,(unsigned long long)m->order,(unsigned long long)m->draws,m->a,m->b,m->c);}
    for(i=0;i<site_count;i++) {site_row *s=&sites[i];ADD("core-crt-site file=%s line=%d pre=%llu load=%llu gap=%llu first_pre=%llu first_load=%llu first_gap=%llu last=%llu first_value=%d last_value=%d gap_scene=%d\n",basename_of(s->file),s->line,(unsigned long long)s->count[0],(unsigned long long)s->count[1],(unsigned long long)s->count[2],(unsigned long long)s->first[0],(unsigned long long)s->first[1],(unsigned long long)s->first[2],(unsigned long long)s->last,s->first_value,s->last_value,s->gap_scene);}
    for(i=0;i<skin_count;i++) {skin_row *s=&skins[i];ADD("core-crt-skin scene=%d animation=%d season=%d normal=%d desert=%d winter=%d ctor=%d frames=%d calls=%llu draws=%llu\n",s->scene,s->animation,s->season,s->normal,s->desert,s->winter,s->ctor,s->frames,(unsigned long long)s->calls,(unsigned long long)s->draws);}
#undef ADD
    return 0;
}
#endif
