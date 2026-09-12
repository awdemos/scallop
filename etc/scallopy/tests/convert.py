import unittest

import scallopy

class TestConversion(unittest.TestCase):
  def test_convert_1(self):
    ctx = scallopy.ScallopContext()
    ctx.add_relation("r", (int, int))
    ctx.add_facts("r", [(1, 3), (2, 4)])
    ctx2 = ctx.clone("topkproofs")
    ctx2.run()
    self.assertEqual(list(ctx2.relation("r")), [(1.0, (1, 3)), (1.0, (2, 4))])

  def test_convert_2(self):
    ctx = scallopy.ScallopContext("topkproofs")
    ctx.add_relation("r", (int, int))
    ctx.add_facts("r", [(0.5, (1, 3)), (0.8, (2, 4))])
    ctx2 = ctx.clone("unit")
    ctx2.run()
    self.assertEqual(list(ctx2.relation("r")), [(1, 3), (2, 4)])

  def test_convert_3(self):
    ctx = scallopy.ScallopContext("topkproofs", k=3)
    ctx2 = ctx.clone("topkproofs")
    self.assertEqual(ctx2._k, 3)

  def test_convert_4(self):
    ctx = scallopy.ScallopContext("topkproofs", k=3)
    ctx2 = ctx.clone("topkproofs", k=5)
    self.assertEqual(ctx2._k, 5)

  def test_convert_5(self):
    ctx = scallopy.ScallopContext("topkproofs")
    ctx2 = ctx.clone("topkproofs", wmc_with_disjunctions=True)
    self.assertTrue(ctx2._wmc_with_disjunctions)

  def test_convert_6(self):
    ctx = scallopy.ScallopContext("topkproofs", wmc_with_disjunctions=True)
    ctx2 = ctx.clone("topkproofs")
    self.assertTrue(ctx2._wmc_with_disjunctions)


if __name__ == "__main__":
  unittest.main()
