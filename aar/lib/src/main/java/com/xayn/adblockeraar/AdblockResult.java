package com.xayn.adblockeraar;

   public class AdblockResult {
       private final static byte IS_MATCHED_MASK = 1;
       private final static byte IS_IMPORTANT_MASK = 2;
       private final static byte IS_EXCEPTION_MASK = 4;

       protected final byte nativeResult;

       public AdblockResult(byte nativeResult) {
           this.nativeResult = nativeResult;
       }

       public boolean isMatched() {
           return (nativeResult & IS_MATCHED_MASK) != 0;
       }

       public boolean isImportant() {
           return (nativeResult & IS_IMPORTANT_MASK) != 0;
       }

       public boolean isException() {
           return (nativeResult & IS_EXCEPTION_MASK) != 0;
       }
   }
