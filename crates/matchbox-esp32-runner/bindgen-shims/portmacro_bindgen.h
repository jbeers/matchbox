#pragma once

#include <stdint.h>

#ifndef PORTMACRO_H
#define PORTMACRO_H

#define portCHAR int8_t
#define portFLOAT float
#define portDOUBLE double
#define portLONG int32_t
#define portSHORT int16_t
#define portSTACK_TYPE uint8_t
#define portBASE_TYPE int

typedef portSTACK_TYPE StackType_t;
typedef portBASE_TYPE BaseType_t;
typedef unsigned portBASE_TYPE UBaseType_t;
typedef uint32_t TickType_t;

typedef struct {
    uint32_t owner;
    uint32_t count;
} portMUX_TYPE;

#define portMAX_DELAY ((TickType_t)0xffffffffUL)
#define portTICK_PERIOD_MS ((TickType_t)1)
#define portBYTE_ALIGNMENT 16
#define portSTACK_GROWTH (-1)
#define portCRITICAL_NESTING_IN_TCB 1
#define portMUX_INITIALIZER_UNLOCKED { 0xB33FFFFF, 0 }
#define portMUX_FREE_VAL 0xB33FFFFF
#define portMUX_NO_TIMEOUT -1
#define portMUX_INITIALIZE(mux)
#define portENTER_CRITICAL(mux)
#define portEXIT_CRITICAL(mux)
#define portENTER_CRITICAL_ISR(mux)
#define portEXIT_CRITICAL_ISR(mux)
#define portYIELD()
#define portYIELD_FROM_ISR(...)
#define portYIELD_WITHIN_API()
#define portDISABLE_INTERRUPTS()
#define portENABLE_INTERRUPTS()
#define portSET_INTERRUPT_MASK_FROM_ISR() 0
#define portCLEAR_INTERRUPT_MASK_FROM_ISR(prev_level)
#define portASSERT_IF_IN_ISR()
#define portCHECK_IF_IN_ISR() 0
#define portGET_CORE_ID() ((BaseType_t)0)
#define portYIELD_CORE(core_id)
#define portTASK_FUNCTION_PROTO(vFunction, pvParameters) void vFunction(void *pvParameters)
#define portTASK_FUNCTION(vFunction, pvParameters) void vFunction(void *pvParameters)

BaseType_t xPortInIsrContext(void);
BaseType_t xPortEnterCriticalTimeout(portMUX_TYPE *mux, BaseType_t timeout);
void vPortExitCritical(portMUX_TYPE *mux);
void _frxt_setup_switch(void);
void vPortYield(void);

#endif
