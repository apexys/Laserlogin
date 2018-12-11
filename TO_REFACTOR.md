1. PRIO MEDIUM Get all the double request guards where _AdminUser and Usertype are requested to only use a single guard by implementing an Into trait on AdminUser, NormalUser,... to get the usertype (which could be stored inside the struct)

2. PRIO LOW Rewrite the software joins I'm currently doing to use SQL queries to slightly improve performance

3. PRIO MEDIUM Look at all envs passed to the templating system and see if all the data passed into it actually gets used or if we can save time by not serializing it to JSON objects first

4. PRIO HIGH Changing the inventory name on an inventory actually creates a new one with that name