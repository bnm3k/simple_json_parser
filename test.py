import unittest

import my_json


class TestJSON(unittest.TestCase):
    def test_parse_from_string(self):
        json_val = '{"foo": 1}'
        got = my_json.parse_from_string(json_val)
        expect = {"foo": 1}
        self.assertEqual(got, expect)


if __name__ == "__main__":
    unittest.main()
