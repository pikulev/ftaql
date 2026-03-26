// 1. Standard alias from the base config, pointing to a file
import { helperFunction } from "@lib/helpers";

// 2. Second import from the same file, to ensure resolution is stable
import { anotherHelper } from "@lib/helpers";

// 3. Alias with fallback paths
import { fallbackFunction } from "@lib/fallback/one";

// 4. App-specific alias from the nested tsconfig.json
import { appUtil } from "@/utils";

// 5. Root alias to a different file
import { utilFunction } from "@lib/utils";

console.log(
  helperFunction(),
  anotherHelper(),
  fallbackFunction(),
  appUtil(),
  utilFunction()
); 