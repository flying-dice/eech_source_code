

































































#include "project.h"





#define DEBUG_MODULE 0





static void qs (entity **en_list, float *sort_order, int left, int right)
{
   int
      i, j;

   float
      x, y;

   entity
      *temp;

   i = left;
   j = right;

   x = sort_order [(left + right) / 2];

   do
   {
      while (sort_order [i] > x && i < right) i++;
      while (x > sort_order [j] && j > left) j--;

      if (i <= j)
      {
         y = sort_order [i];
         sort_order [i] = sort_order [j];
         sort_order [j] = y;

         temp = en_list [i];
         en_list [i] = en_list [j];
         en_list [j] = temp;
         
         i++;
         j--;
      }
   }
	while (i <= j);

   if (left < j) qs (en_list, sort_order, left, j);
   if (i < right) qs (en_list, sort_order, i, right);
}





void quicksort_entity_list (entity **en_list, int count, float *sort_order)
{
	ASSERT (en_list);

   qs (en_list, sort_order, 0, count - 1);
}






