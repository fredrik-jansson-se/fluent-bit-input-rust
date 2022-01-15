#include <fluent-bit.h>

extern int cb_init(struct flb_input_instance *, struct flb_config *, void *);
extern int cb_collect (struct flb_input_instance *, struct flb_config *, void *);
extern int cb_exit (void *, struct flb_config *);

static struct flb_config_map config_map[] = {
   {
    FLB_CONFIG_MAP_INT, "interval_sec", "30",
    0, FLB_FALSE, 0,
    "Collect interval."
   },
   {0}
};

extern struct flb_input_plugin in_example_plugin;

struct flb_input_plugin in_example_plugin = {
    .name         = "example",
    .description  = "Example log input plugin",
    .event_type   = FLB_INPUT_LOGS,
    .cb_init      = cb_init,
    .cb_pre_run   = NULL,
    .cb_collect   = cb_collect,
    .cb_flush_buf = NULL,
    .config_map   = config_map,
    .cb_exit      = cb_exit,
};
