#pragma once
#include <fluent-bit/flb_input.h>

extern enum flb_loglevel_helper {
   LOG_LVL_OFF = FLB_LOG_OFF, 
   LOG_LVL_ERROR  = FLB_LOG_ERROR,
   LOG_LVL_WARN = FLB_LOG_WARN,
   LOG_LVL_INFO = FLB_LOG_INFO,
   LOG_LVL_DEBUG = FLB_LOG_DEBUG,
   LOG_LVL_TRACE = FLB_LOG_TRACE,
};
