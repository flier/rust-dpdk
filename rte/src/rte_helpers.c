#include <stdio.h>
#include <stdlib.h>

#include <rte_errno.h>

int _rte_errno()
{
    return rte_errno;
}
