Where next??

I need to alter the behaviour of training_run.
I need to re-examine the way perfect scores are handled.
At the moment a perfect score is "prevented" from happing by adding a tiny value, e0, to the score.
I this it would be better to handle the perfect score case explicitly, either by allowing and catering for infinity or, perhaps better, by having an enum type that can be either a finite score or perfect (this would force the handling of perfect scores to be explicit).

I also need to re-examine the behaviour when the resolution limit is reached.
The current behaviour is to return the best score found so far, but perhaps it would be better to return a special value indicating that the resolution limit was reached.
However, it is possible that the hitting of the resolution limit is just a temporary state and while it does indicate that there is nothing to be gained by dividing the current cells it doesn't necessarily mean that there is no point in continuing training.

Once I have some reliable long term runs and need to experiment with different algorithms for reproduction and mutation.

Reproduction currently is very imperfect, matching parents from their order in the sorted list of scores.
This leads to a high probability of very closely related parents being chosen.
There is evidence that natural systems go to great lengths to avoid this situation which suggests that it is sub-optimal.
I need to implement a more sophisticated scheme perhaps that looks at an organisms parents to avoid close relatives.
It may even be beneficial to store grandparents also.

Also there are currently no random mutations of the genome.
A random mutation in theory allows the system to escape local optima, at the cost of a slower convergence rate.
I think that this may be beneficial, but I need to experiment to see if this is the case.
