# How payments will work

## Gifts

If the gifted price is a recurring one:

- Convert current sub to subscription schedule if it isn't already
- Append a new phase to the schedule with the gifted price and a 100% discount
- Append a new phase that's the same as the previous one (if there was one)
