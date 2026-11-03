

































































#define FOG_OF_WAR_DECAY_RATE		(30.0)

#define DEFAULT_FOG_OF_WAR_MAXIMUM_VALUE	(4.0 * ONE_HOUR)





extern void create_patrol_task(entity *group, entity *keysite, entity_sides side);

extern void initialise_high_level_ai (void);

extern void deinitialise_high_level_ai (void);

extern void start_high_level_ai ();

extern void stop_high_level_ai ();

extern int create_sead_tasks_around_keysite (entity *original_task, entity *keysite, entity_sides this_side);





#ifdef DEBUG

extern void ai_log (const char *string, ...);

#else

#ifdef WIN32
#define ai_log();
#else
#define ai_log(a, x...) do { } while(0);
#endif

#endif





#include "imaps.h"

#include "order.h"

#include "reaction.h"

#include "setup.h"

#include "suitable.h"




