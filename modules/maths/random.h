









































































extern int
	random_number_seed;





extern void initialise_random_number_generator (void);

extern void set_random_number_seed (int seed);

extern int get_random_number_seed (void);





//
// get random number from system seed
//

#define get_random_number() (random_number_seed = ((random_number_seed * 1103515245) + 12345))

//
// get random number from user seed (pass int pointer to seed)
//

#define get_random_number_user(SEED) ((int) (*(SEED) = ((*(SEED) * 1103515245) + 12345)))

//
// value = rand () & 0x7F;
//

#define rand8() ((get_random_number () >> 16) & 0x7F)

#define rand8x(SEED) ((get_random_number_user ((SEED)) >> 16) & 0x7F)

//
// value = rand ();
//

#define rand16() ((get_random_number () >> 16) & 0x7FFF)

#define rand16x(SEED) ((get_random_number_user ((SEED)) >> 16) & 0x7FFF)


//
// WARNING: rand32 () will always return an even number
//


//
// value = rand () << 16;
//

#define rand32() (get_random_number () & 0x7FFF0000)

#define rand32x(SEED) (get_random_number_user ((SEED)) & 0x7FFF0000)





//
// get random value in range -1.0 to 1.0
//

#define sfrand1() (((double) (rand16 () - 16384)) * (1.0 / 16384.0))

#define sfrand1x(SEED) (((double) (rand16x ((SEED)) - 16384)) * (1.0 / 16384.0))

// normalized value (higher probability of close to zero
double sfrand1norm();

//
// get random value in range 0.0 to 1.0
//

#define frand1() (((double) rand16 ()) * (1.0 / 32767.0))

#define frand1x(SEED) (((double) rand16x ((SEED))) * (1.0 / 32767.0))




